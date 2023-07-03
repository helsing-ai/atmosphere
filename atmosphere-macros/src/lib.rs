use proc_macro::{self, Span, TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use sqlx::{Postgres, QueryBuilder};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    parse_macro_input, parse_quote, Attribute, Data, DataStruct, DeriveInput, Expr, ExprLit, Field,
    Fields, FieldsNamed, Ident, Lifetime, Lit, LitStr, Meta, MetaNameValue,
};

#[proc_macro_derive(Model, attributes(id, reference))]
pub fn model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed {
            named: columns,
            ..
        }),
        ..
    }) = &input.data else {
        panic!("only named structs can derive the model trait");
    };

    let model = Model::parse(&input, &columns);

    let model_trait_impl = model.quote_trait_impl();
    let read_impl = model.quote_read_impl();
    let write_impl = model.quote_write_impl();

    quote! {
        #model_trait_impl
        #read_impl
        #write_impl
    }
    .into()
}

#[derive(Clone)]
struct Model {
    ident: Ident,
    schema: String,
    table: String,
    id: Column,
    refs: Vec<Reference>,
    data: Vec<Column>,
}

impl Model {
    fn parse(input: &DeriveInput, fields: &Punctuated<Field, Comma>) -> Self {
        let ident = &input.ident;

        let columns = fields.iter().map(Column::parse);

        let (id, data): (Vec<Column>, Vec<Column>) = columns.partition(|c| c.id);

        let id = {
            if id.len() == 0 {
                panic!("missing primary id column (#[id]) on model {}", ident);
            }
            if id.len() > 1 {
                panic!(
                    "found more than one primary id column (#[id]) on model {}",
                    ident
                );
            }
            id[0].clone()
        };

        let data = data.into_iter().filter(|d| !d.fk).collect();
        let refs: Vec<Reference> = fields.iter().filter_map(Reference::parse).collect();

        Self {
            ident: ident.to_owned(),
            schema: "public".to_owned(),
            table: ident.to_string().to_lowercase(),
            id,
            refs,
            data,
        }
    }

    fn quote_trait_impl(&self) -> TokenStream2 {
        let Self {
            ident,
            schema,
            table,
            id,
            refs,
            data,
        } = self;

        let id_ty = &self.id.ty;
        let id = self.id.quote();
        let refs = self.refs.iter().map(|r| r.column.quote());
        let data = self.data.iter().map(|d| d.quote());

        quote!(
            #[automatically_derived]
            impl ::atmosphere::Model for #ident {
                type Id = #id_ty;

                const ID: ::atmosphere::Column<#ident> = #id;

                const SCHEMA: &'static str = #schema;
                const TABLE: &'static str = #table;

                const REFS: &'static [::atmosphere::Column<#ident>] = &[
                    #(#refs),*
                ];
                const DATA: &'static [::atmosphere::Column<#ident>] = &[
                    #(#data),*
                ];
            }
        )
    }

    fn quote_read_impl(&self) -> TokenStream2 {
        let Self {
            ident,
            schema,
            table,
            id,
            refs,
            data,
        } = self;

        let all = self.select().into_sql();

        let find = {
            let mut query = self.select();
            query.push(format!("WHERE\n  {} = $1", id.name));
            query.into_sql()
        };

        quote!(
            #[automatically_derived]
            #[::atmosphere::prelude::async_trait]
            impl ::atmosphere::Read for #ident {
                async fn find(id: &Self::Id, pool: &::sqlx::PgPool) -> ::atmosphere_core::Result<Self> {
                    ::sqlx::query_as!(
                        Self,
                        #find,
                        id
                    )
                    .fetch_one(pool)
                    .await
                    .map_err(|_| ())
                }

                async fn all(pool: &::sqlx::PgPool) -> ::atmosphere_core::Result<Vec<Self>> {
                    ::sqlx::query_as!(
                        Self,
                        #all
                    )
                    .fetch_all(pool)
                    .await
                    .map_err(|_| ())
                }
            }
        )
    }

    fn quote_write_impl(&self) -> TokenStream2 {
        let Self {
            ident,
            schema,
            table,
            id,
            refs,
            data,
        } = self;

        let field_bindings = {
            let mut fields = vec![];

            let name = &id.name;
            fields.push(quote!(&self.#name as _));

            for r in refs {
                let name = &r.column.name;
                fields.push(quote!(&self.#name as _));
            }

            for d in data {
                let name = &d.name;
                fields.push(quote!(&self.#name as _));
            }

            fields
        };

        let save = self.insert().into_sql();

        let update = {
            let mut query = self.update();
            query.push(format!("\nWHERE\n  {} = $1", id.name));
            query.into_sql()
        };

        let delete = {
            let mut query = self.delete();
            query.push(format!("\nWHERE\n  {} = $1", id.name));
            query.into_sql()
        };

        let delete_field = {
            let name = &id.name;

            quote!(&self.#name)
        };

        dbg!(&save, &update, &delete);

        quote!(
            #[automatically_derived]
            #[::atmosphere::prelude::async_trait]
            impl ::atmosphere::Write for #ident {
                async fn save(&self, pool: &::sqlx::PgPool) -> ::atmosphere_core::Result<()> {
                    ::sqlx::query!(
                        #save,
                        #(#field_bindings),*
                    )
                    .execute(pool)
                    .await
                    .map(|_| ())
                    .map_err(|_| ())
                }

                async fn update(&self, pool: &::sqlx::PgPool) -> ::atmosphere_core::Result<()> {
                    ::sqlx::query!(
                        #update,
                        #(#field_bindings),*
                    )
                    .execute(pool)
                    .await
                    .map(|_| ())
                    .map_err(|_| ())
                }

                async fn delete(&self, pool: &::sqlx::PgPool) -> ::atmosphere_core::Result<()> {
                    ::sqlx::query!(
                        #delete,
                        #delete_field
                    )
                    .execute(pool)
                    .await
                    .map(|_| ())
                    .map_err(|_| ())
                }
            }
        )
    }
}

/// Query Building Related Operations
impl Model {
    fn escaped_table(&self) -> String {
        format!("\"{}\".\"{}\"", self.schema, self.table)
    }

    /// Generate the base select statement
    fn select(&self) -> sqlx::QueryBuilder<sqlx::Postgres> {
        let Self {
            ident,
            schema,
            table,
            id,
            refs,
            data,
        } = self;

        let mut query = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT\n");

        let mut separated = query.separated(",\n  ");

        separated.push(format!("  {} as \"{}: _\"", id.name, id.name));

        for r in refs {
            separated.push(format!("{} as \"{}: _\"", r.column.name, r.column.name));
        }

        for data in data {
            separated.push(format!("{} as \"{}: _\"", data.name, data.name));
        }

        query.push(format!("\nFROM\n  {}\n", self.escaped_table()));

        query
    }

    /// Generate the update statement
    fn update(&self) -> QueryBuilder<Postgres> {
        let Self {
            ident,
            schema,
            table,
            id,
            refs,
            data,
        } = self;

        let mut query =
            QueryBuilder::<Postgres>::new(format!("UPDATE {} SET\n  ", self.escaped_table()));

        let mut separated = query.separated(",\n  ");

        let mut col = 2;

        for r in refs {
            separated.push(format!("{} = ${col}", r.column.name));
            col += 1;
        }

        for data in data {
            separated.push(format!("{} = ${col}", data.name));
            col += 1;
        }

        query
    }

    /// Generate the insert statement
    fn insert(&self) -> QueryBuilder<Postgres> {
        let Self {
            ident,
            schema,
            table,
            id,
            refs,
            data,
        } = self;

        let mut query =
            QueryBuilder::<Postgres>::new(format!("INSERT INTO {} (\n  ", self.escaped_table()));

        let mut separated = query.separated(",\n  ");

        separated.push(id.name.to_string());

        for r in refs {
            separated.push(r.column.name.to_string());
        }

        for data in data {
            separated.push(data.name.to_string());
        }

        separated.push_unseparated("\n) VALUES (\n");

        separated.push_unseparated("  $1");

        let cols = 1 + refs.len() + data.len();

        for c in 2..=cols {
            separated.push(format!("${c}"));
        }

        separated.push_unseparated(")");

        query
    }

    /// Generate the delete statement without where clause
    fn delete(&self) -> QueryBuilder<Postgres> {
        QueryBuilder::<Postgres>::new(format!("DELETE FROM {}", self.escaped_table()))
    }
}

#[derive(Clone)]
struct Column {
    id: bool,
    fk: bool,
    name: Ident,
    ty: syn::Type,
}

impl Column {
    fn parse(field: &Field) -> Self {
        let id = field.attrs.iter().any(|a| a.path.is_ident("id"));
        let fk = field.attrs.iter().any(|a| a.path.is_ident("reference"));

        if id && fk {
            panic!(
                "{} can not be primary key and foreign key at the same time",
                field.ident.as_ref().unwrap()
            );
        }

        Self {
            id,
            fk,
            name: field.ident.clone().unwrap(),
            ty: field.ty.clone(),
        }
    }

    fn quote(&self) -> TokenStream2 {
        let Column { id, fk, name, ty } = self;

        let name = name.to_string();

        let col_type = if *id {
            quote!(::atmosphere_core::ColType::PrimaryKey)
        } else if *fk {
            quote!(::atmosphere_core::ColType::ForeignKey)
        } else {
            quote!(::atmosphere_core::ColType::Value)
        };

        quote!(
            ::atmosphere_core::Column::new(
                #name,
                ::atmosphere_core::DataType::Unknown,
                #col_type
            )
        )
    }
}

#[derive(Clone)]
struct Reference {
    model: Ident,
    column: Column,
}

impl Reference {
    fn parse(field: &Field) -> Option<Self> {
        let referenced = field
            .attrs
            .iter()
            .filter(|a| a.path.is_ident("reference"))
            .map(|a| {
                a.parse_args::<Ident>()
                    .expect("ref requires the model it refers to as parameter")
            })
            .collect::<Vec<Ident>>();

        let model = referenced.get(0)?;

        Some(Self {
            model: model.to_owned(),
            column: Column::parse(&field),
        })
    }
}

// Query Macros
#[proc_macro_attribute]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    println!("attr: \"{}\"", attr.to_string());
    println!("item: \"{}\"", item.to_string());

    let mut query = parse_macro_input!(item as syn::ItemFn);

    let pool: syn::FnArg = parse_quote!(pool: &::sqlx::PgPool);
    query.sig.inputs.push(pool);

    let (one, many): (syn::Type, syn::Type) = (parse_quote!(Self), parse_quote!(Vec<Self>));

    let fetch = match query.sig.output {
        syn::ReturnType::Type(_, ref one) => quote!(fetch_one(pool)),
        syn::ReturnType::Type(_, ref many) => quote!(fetch_many(pool)),
        _ => panic!("unsupported return type found, only `Self` and `Vec<Self>` are supported"),
    };

    let block = query.block;
    query.block = parse_quote!({
        Ok(#block.#fetch.await.unwrap())
    });

    quote!(#query).into()
}

#[proc_macro]
pub fn select(input: TokenStream) -> TokenStream {
    let raw = input.to_string();

    let sql = raw.split(" ");
    let mut sanitized = String::new();
    let mut args: Vec<Ident> = vec![];

    for word in sql {
        if word.starts_with("$") {
            let arg: String = word.chars().skip(1).collect();
            args.push(Ident::new(&arg, Span::call_site().into()));
            sanitized.push_str(&format!(" ${}", args.len()));
            continue;
        }

        sanitized.push_str(&format!(" {word}"));
    }

    let query = format!("select * from public.forest {sanitized}");

    dbg!(&query);

    quote!(::sqlx::query_as!(
        Self,
        #query,
        #(&#args),*
    ))
    .into()
}

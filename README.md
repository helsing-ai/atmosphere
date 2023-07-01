<div align="center">

# `🌍 Atmosphere`

**A lightweight sqlx framework for safe and fast postgres interactions**

[![Hemisphere](https://img.shields.io/badge/hemisphere-open%20source-blueviolet.svg)](https://hemisphere.studio)
[![Hemisphere](https://img.shields.io/badge/postgresql-orm-blue.svg)]()

</div>

## Roadmap

### Alpha Release
- [x] Trait System (`Model`, `Column`, `Reference` ..)
- [x] Derive Macro (`Model`)
- [x] Field Attributes (`#[id]`, and so on)
- [x] Query Generation

### Beta Release
- [ ] Implement all standard queries (`find`, `store`, `update`)

### Stable Release
- [ ] Stabilize Traits
- [ ] Filtering and `where` macro
- [ ] Model Views (subsets)

### Advanced
- [ ] Compile Time Verification
- [ ] Postgres Composite Types
- [ ] Support `updated_at` fields
- [ ] Support custom types
- [ ] Runtime Inspection
- [ ] Generate Graphs
- [ ] `validator` support

### Longterm
- [ ] Generate HTTP, gRPC and GraphQL Bindings

## Quickstart

### Schema

```sql
CREATE TABLE recipe (
    id         UUID PRIMARY KEY,
    name       TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE recipe_step (
    id          UUID PRIMARY KEY,
    recipe_id   UUID NOT NULL REFERENCES recipe(id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    duration    INTERVAL NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL
);
```

### Model

```rust
use atmosphere::prelude::*;

#[derive(Model)]
#[atmosphere(table = "recipe")]
struct Recipe {
    #[id]
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>
}

impl Recipe {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new(),
            name,
            created_at: Utc::now()
        }
    }
}

#[derive(Model)]
#[atmosphere(table = "recipe_step")]
struct Step {
    #[id]
    pub id: Uuid,
    #[ref(Recipe)]
    pub recipe_id: Uuid,
    pub description: String,
    pub duration: Duration,
    pub created_at: DateTime<Utc>
}

impl Step {
    pub fn new(recipe: &Recipe, description: String, duration: Duration) -> Self {
        Self {
            id: Uuid::new(),
            recipe_id: *recipe.id,
            description,
            duration,
            created_at: Utc::now()
        }
    }
}
```

### Usage

```rust
use atmosphere::prelude::*;

async fn main() -> atmosphere::Result<()> {
    let pool = /* sqlx postgres connection */;

    // Create
    let cheesecake = {
        let cheesecake = Recipe::new("cheesecake".to_string());
        cheesecake.save(&pool).await?;

        Step::new(&cheesecake, "Bake it".to_owned(), Duration::minutes(60))
            .save(&pool)
            .await?;

        cheesecake
    };

    let apple_pie = {
        let apple_pie = Recipe::new("Apple Pie".to_string());
        apple_pie.save(&pool).await?;

        Step::new(&apple_pie, "Bake it".to_owned(), Duration::minutes(45))
            .save(&pool)
            .await?;

        apple_pie
    };

    // Get
    assert_eq!(Recipe::get(&apple_pie.id, &pool).await?, apple_pie);
    assert_eq!(Step::for<Recipe>(&apple_pie.id, &pool).await?.len(), 1);

    // List
    let all = Recipe::list(&pool).await?;
    assert_eq!(all, vec![cheesecake, apple_pie]);

    // Filter
    let filtered = Recipe::select(where! {
            created_at > cheesecake.created_at
        })
        .await?;
    assert_eq!(filtered, vec![apple_pie]);

    // Update
    let mut carrot_cake = cheesecake;
    carrot_cake.name = "Carrot Cake".to_string();
    carrot_cake.save(&pool).await;
    drop(cheesecake);

    // Delete
    apple_pie.delete(&pool).await?;
}
```

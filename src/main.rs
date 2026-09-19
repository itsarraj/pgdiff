use clap::Parser;
use pgdiff::{diff, introspect, sql};
use sqlx::postgres::PgPoolOptions;

#[derive(Parser)]
#[command(
    name = "pgdiff",
    about = "Compares two live Postgres schemas and generates the SQL to reconcile them"
)]
struct Cli {
    old_database_url: String,
    new_database_url: String,
    #[arg(long, default_value = "public")]
    schema: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let old_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&cli.old_database_url)
        .await?;
    let new_pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&cli.new_database_url)
        .await?;

    let old_schema = introspect::introspect(&old_pool, &cli.schema).await?;
    let new_schema = introspect::introspect(&new_pool, &cli.schema).await?;

    let changes = diff::diff_schemas(&old_schema, &new_schema);
    if changes.is_empty() {
        println!("-- no differences found");
    } else {
        println!("-- {} change(s) to go from old -> new:", changes.len());
        for change in &changes {
            println!("{}", sql::render(change));
        }
    }
    Ok(())
}

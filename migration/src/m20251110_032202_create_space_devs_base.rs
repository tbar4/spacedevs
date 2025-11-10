//! 2023‑01‑01 00:00:01 – Create the full Spaceflight‑News data model
//! --------------------------------------------------------------
//! This migration covers every entity that appears in the SeaORM data model
//! you asked for:
//!   * articles / blogs / reports (identical columns)
//!   * authors + author_socials
//!   * launches + events (simple reference tables)
//!   * all many‑to‑many join tables (article_* , blog_*, report_*)
//! It deliberately **ignores** the top‑level pagination fields
//! (`count`, `next`, `previous`).

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Helper to avoid repetition for the three “content” tables.
fn content_table(name: &str) -> TableCreateStatement {
    Table::create()
        .if_not_exists()
        .table(Alias::new(name))
        .col(
            ColumnDef::new(Alias::new("id"))
                .integer()
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new(Alias::new("title")).string().not_null())
        .col(ColumnDef::new(Alias::new("url")).string().null())
        .col(ColumnDef::new(Alias::new("image_url")).string().null())
        .col(ColumnDef::new(Alias::new("news_site")).string().null())
        .col(ColumnDef::new(Alias::new("summary")).text().null())
        .col(
            ColumnDef::new(Alias::new("published_at"))
                .date_time()
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("updated_at"))
                .date_time()
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("featured"))
                .boolean()
                .not_null()
                .default(false),
        )
        .to_owned()
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// --------------------------------------------------------------------
    /// `up` – create every table, indexes and foreign‑key constraints.
    /// --------------------------------------------------------------------
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ----------------------------------------------------------------
        // 1️⃣  Shared lookup tables
        // ----------------------------------------------------------------
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(Authors::Table)
                    .col(
                        ColumnDef::new(Authors::Id)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Authors::Name).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(AuthorSocials::Table)
                    .col(
                        ColumnDef::new(AuthorSocials::AuthorId)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuthorSocials::X).string().null())
                    .col(ColumnDef::new(AuthorSocials::Youtube).string().null())
                    .col(ColumnDef::new(AuthorSocials::Instagram).string().null())
                    .col(ColumnDef::new(AuthorSocials::Linkedin).string().null())
                    .col(ColumnDef::new(AuthorSocials::Mastodon).string().null())
                    .col(ColumnDef::new(AuthorSocials::Bluesky).string().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-author_socials-author_id")
                            .from(AuthorSocials::Table, AuthorSocials::AuthorId)
                            .to(Authors::Table, Authors::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(Launches::Table)
                    .col(
                        ColumnDef::new(Launches::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Launches::ExternalId).big_integer().null())
                    .col(ColumnDef::new(Launches::Name).string().null())
                    .col(ColumnDef::new(Launches::Provider).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(Events::Table)
                    .col(
                        ColumnDef::new(Events::Id)
                            .integer()
                            .not_null()
                            .primary_key()
                            .auto_increment(),
                    )
                    .col(ColumnDef::new(Events::ExternalId).big_integer().null())
                    .col(ColumnDef::new(Events::Name).string().null())
                    .col(ColumnDef::new(Events::Provider).string().null())
                    .to_owned(),
            )
            .await?;

        // ----------------------------------------------------------------
        // 2️⃣  Content tables (articles, blogs, reports) – identical schema
        // ----------------------------------------------------------------
        manager.create_table(content_table("articles")).await?;
        manager.create_table(content_table("blogs")).await?;
        manager.create_table(content_table("reports")).await?;

        // ----------------------------------------------------------------
        // 3️⃣  Join tables – composite primary keys + FKs
        // ----------------------------------------------------------------
        // ---- article ↔ author -------------------------------------------------
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(ArticleAuthors::Table)
                    .col(
                        ColumnDef::new(ArticleAuthors::ArticleId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ArticleAuthors::AuthorId)
                            .integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(ArticleAuthors::ArticleId)
                            .col(ArticleAuthors::AuthorId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-article_authors-article_id")
                            .from(ArticleAuthors::Table, ArticleAuthors::ArticleId)
                            .to(Articles::Table, Articles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-article_authors-author_id")
                            .from(ArticleAuthors::Table, ArticleAuthors::AuthorId)
                            .to(Authors::Table, Authors::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ---- blog ↔ author ----------------------------------------------------
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(BlogAuthors::Table)
                    .col(ColumnDef::new(BlogAuthors::BlogId).integer().not_null())
                    .col(ColumnDef::new(BlogAuthors::AuthorId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(BlogAuthors::BlogId)
                            .col(BlogAuthors::AuthorId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-blog_authors-blog_id")
                            .from(BlogAuthors::Table, BlogAuthors::BlogId)
                            .to(Blogs::Table, Blogs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-blog_authors-author_id")
                            .from(BlogAuthors::Table, BlogAuthors::AuthorId)
                            .to(Authors::Table, Authors::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ---- report ↔ author --------------------------------------------------
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(ReportAuthors::Table)
                    .col(ColumnDef::new(ReportAuthors::ReportId).integer().not_null())
                    .col(ColumnDef::new(ReportAuthors::AuthorId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(ReportAuthors::ReportId)
                            .col(ReportAuthors::AuthorId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-report_authors-report_id")
                            .from(ReportAuthors::Table, ReportAuthors::ReportId)
                            .to(Reports::Table, Reports::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-report_authors-author_id")
                            .from(ReportAuthors::Table, ReportAuthors::AuthorId)
                            .to(Authors::Table, Authors::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ---- article ↔ launch -------------------------------------------------
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(ArticleLaunches::Table)
                    .col(
                        ColumnDef::new(ArticleLaunches::ArticleId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ArticleLaunches::LaunchId)
                            .integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(ArticleLaunches::ArticleId)
                            .col(ArticleLaunches::LaunchId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-article_launches-article_id")
                            .from(ArticleLaunches::Table, ArticleLaunches::ArticleId)
                            .to(Articles::Table, Articles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-article_launches-launch_id")
                            .from(ArticleLaunches::Table, ArticleLaunches::LaunchId)
                            .to(Launches::Table, Launches::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ---- blog ↔ launch ----------------------------------------------------
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(BlogLaunches::Table)
                    .col(ColumnDef::new(BlogLaunches::BlogId).integer().not_null())
                    .col(ColumnDef::new(BlogLaunches::LaunchId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(BlogLaunches::BlogId)
                            .col(BlogLaunches::LaunchId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-blog_launches-blog_id")
                            .from(BlogLaunches::Table, BlogLaunches::BlogId)
                            .to(Blogs::Table, Blogs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-blog_launches-launch_id")
                            .from(BlogLaunches::Table, BlogLaunches::LaunchId)
                            .to(Launches::Table, Launches::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ---- report ↔ launch --------------------------------------------------
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(ReportLaunches::Table)
                    .col(
                        ColumnDef::new(ReportLaunches::ReportId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ReportLaunches::LaunchId)
                            .integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(ReportLaunches::ReportId)
                            .col(ReportLaunches::LaunchId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-report_launches-report_id")
                            .from(ReportLaunches::Table, ReportLaunches::ReportId)
                            .to(Reports::Table, Reports::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-report_launches-launch_id")
                            .from(ReportLaunches::Table, ReportLaunches::LaunchId)
                            .to(Launches::Table, Launches::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ---- article ↔ event --------------------------------------------------
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(ArticleEvents::Table)
                    .col(
                        ColumnDef::new(ArticleEvents::ArticleId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ArticleEvents::EventId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(ArticleEvents::ArticleId)
                            .col(ArticleEvents::EventId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-article_events-article_id")
                            .from(ArticleEvents::Table, ArticleEvents::ArticleId)
                            .to(Articles::Table, Articles::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-article_events-event_id")
                            .from(ArticleEvents::Table, ArticleEvents::EventId)
                            .to(Events::Table, Events::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ---- blog ↔ event -----------------------------------------------------
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(BlogEvents::Table)
                    .col(ColumnDef::new(BlogEvents::BlogId).integer().not_null())
                    .col(ColumnDef::new(BlogEvents::EventId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(BlogEvents::BlogId)
                            .col(BlogEvents::EventId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-blog_events-blog_id")
                            .from(BlogEvents::Table, BlogEvents::BlogId)
                            .to(Blogs::Table, Blogs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-blog_events-event_id")
                            .from(BlogEvents::Table, BlogEvents::EventId)
                            .to(Events::Table, Events::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ---- report ↔ event ---------------------------------------------------
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(ReportEvents::Table)
                    .col(ColumnDef::new(ReportEvents::ReportId).integer().not_null())
                    .col(ColumnDef::new(ReportEvents::EventId).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(ReportEvents::ReportId)
                            .col(ReportEvents::EventId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-report_events-report_id")
                            .from(ReportEvents::Table, ReportEvents::ReportId)
                            .to(Reports::Table, Reports::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-report_events-event_id")
                            .from(ReportEvents::Table, ReportEvents::EventId)
                            .to(Events::Table, Events::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ----------------------------------------------------------------
        // Optional: some handy indexes for look‑ups (Speeds up queries)
        // ----------------------------------------------------------------
        manager
            .create_index(
                Index::create()
                    .name("idx-article_authors-author_id")
                    .table(ArticleAuthors::Table)
                    .col(ArticleAuthors::AuthorId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx-blog_authors-author_id")
                    .table(BlogAuthors::Table)
                    .col(BlogAuthors::AuthorId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx-report_authors-author_id")
                    .table(ReportAuthors::Table)
                    .col(ReportAuthors::AuthorId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    /// --------------------------------------------------------------------
    /// `down` – drop everything in reverse dependency order.
    /// --------------------------------------------------------------------
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop join tables first (they depend on the FK tables)
        manager
            .drop_table(Table::drop().table(ReportEvents::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BlogEvents::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ArticleEvents::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ReportLaunches::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BlogLaunches::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ArticleLaunches::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ReportAuthors::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BlogAuthors::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ArticleAuthors::Table).to_owned())
            .await?;

        // Content tables
        manager
            .drop_table(Table::drop().table(Reports::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Blogs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Articles::Table).to_owned())
            .await?;

        // Look‑up tables
        manager
            .drop_table(Table::drop().table(Events::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Launches::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AuthorSocials::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Authors::Table).to_owned())
            .await?;

        Ok(())
    }
}

/* -------------------------------------------------------------------------
Enum definitions – they give us compile‑time safety for column names.
(SeaORM’s migration tutorial uses the same pattern.)
------------------------------------------------------------------------ */
#[derive(Iden)]
enum Articles {
    Table,
    Id,
}
#[derive(Iden)]
enum Blogs {
    Table,
    Id,
}
#[derive(Iden)]
enum Reports {
    Table,
    Id,
}
#[derive(Iden)]
enum Authors {
    Table,
    Id,
    Name,
}
#[derive(Iden)]
enum AuthorSocials {
    Table,
    AuthorId,
    X,
    Youtube,
    Instagram,
    Linkedin,
    Mastodon,
    Bluesky,
}
#[derive(Iden)]
enum Launches {
    Table,
    Id,
    ExternalId,
    Name,
    Provider,
}
#[derive(Iden)]
enum Events {
    Table,
    Id,
    ExternalId,
    Name,
    Provider,
}

/* ----- Join tables ------------------------------------------------------ */
#[derive(Iden)]
enum ArticleAuthors {
    Table,
    ArticleId,
    AuthorId,
}
#[derive(Iden)]
enum BlogAuthors {
    Table,
    BlogId,
    AuthorId,
}
#[derive(Iden)]
enum ReportAuthors {
    Table,
    ReportId,
    AuthorId,
}
#[derive(Iden)]
enum ArticleLaunches {
    Table,
    ArticleId,
    LaunchId,
}
#[derive(Iden)]
enum BlogLaunches {
    Table,
    BlogId,
    LaunchId,
}
#[derive(Iden)]
enum ReportLaunches {
    Table,
    ReportId,
    LaunchId,
}
#[derive(Iden)]
enum ArticleEvents {
    Table,
    ArticleId,
    EventId,
}
#[derive(Iden)]
enum BlogEvents {
    Table,
    BlogId,
    EventId,
}
#[derive(Iden)]
enum ReportEvents {
    Table,
    ReportId,
    EventId,
}

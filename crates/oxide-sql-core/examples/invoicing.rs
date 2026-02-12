//! Invoicing System - Type-Safe Schema Example
//!
//! This example demonstrates a complete invoicing system with:
//! - Multi-tenant architecture (multiple companies)
//! - Multi-currency support
//! - Client management
//! - Invoice lifecycle (draft -> sent -> paid/overdue)
//!
//! Run with: cargo run --example invoicing

use oxide_sql_core::builder::{Select, col};
use oxide_sql_derive::Table;

// =============================================================================
// SCHEMA DEFINITIONS
// =============================================================================

/// Company - the tenant in a multi-tenant invoicing system.
#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "companies")]
pub struct Company {
    #[column(primary_key)]
    pub id: i64,
    pub name: String,
    #[column(nullable)]
    pub tax_id: Option<String>,
    pub default_currency: String,
    #[column(nullable)]
    pub address: Option<String>,
    pub created_at: String,
}

/// Client - customers who receive invoices.
#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "clients")]
pub struct Client {
    #[column(primary_key)]
    pub id: i64,
    pub company_id: i64,
    pub name: String,
    #[column(nullable)]
    pub email: Option<String>,
    #[column(nullable)]
    pub tax_id: Option<String>,
    pub preferred_currency: String,
    #[column(nullable)]
    pub payment_terms_days: Option<i64>,
    pub created_at: String,
}

/// Invoice - the main billing document.
#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "invoices")]
pub struct Invoice {
    #[column(primary_key)]
    pub id: i64,
    pub company_id: i64,
    pub client_id: i64,
    pub invoice_number: String,
    pub status: String,
    pub currency: String,
    pub subtotal_cents: i64,
    pub tax_rate_percent: f64,
    pub tax_amount_cents: i64,
    pub total_cents: i64,
    pub issue_date: String,
    pub due_date: String,
    #[column(nullable)]
    pub paid_at: Option<String>,
    #[column(nullable)]
    pub notes: Option<String>,
    pub created_at: String,
}

/// InvoiceLine - individual items on an invoice.
#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "invoice_lines")]
pub struct InvoiceLine {
    #[column(primary_key)]
    pub id: i64,
    pub invoice_id: i64,
    pub description: String,
    pub quantity: f64,
    pub unit_price_cents: i64,
    pub total_cents: i64,
    pub sort_order: i64,
}

/// Payment - records of payments received.
#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "payments")]
pub struct Payment {
    #[column(primary_key)]
    pub id: i64,
    pub invoice_id: i64,
    pub amount_cents: i64,
    pub currency: String,
    pub payment_method: String,
    #[column(nullable)]
    pub reference: Option<String>,
    pub paid_at: String,
}

/// ExchangeRate - for multi-currency support.
#[allow(dead_code)]
#[derive(Debug, Clone, Table)]
#[table(name = "exchange_rates")]
pub struct ExchangeRate {
    #[column(primary_key)]
    pub id: i64,
    pub from_currency: String,
    pub to_currency: String,
    pub rate: f64,
    pub effective_date: String,
}

// =============================================================================
// HELPER TO PRINT SQL
// =============================================================================

fn print_sql(description: &str, sql: &str) {
    println!("-- {}", description);
    println!("{};", sql);
    println!();
}

// =============================================================================
// EXAMPLE QUERIES
// =============================================================================

fn main() {
    println!("-- =============================================================================");
    println!("-- INVOICING SYSTEM - SQL QUERIES");
    println!("-- =============================================================================");
    println!();

    let company_id = 1_i64;
    let today = "2024-01-15";

    // -------------------------------------------------------------------------
    // DASHBOARD QUERIES
    // -------------------------------------------------------------------------
    println!("-- Dashboard Queries");
    println!("-- -----------------");
    println!();

    // 1. Get all invoices for the current month
    let (sql, _) = Select::<InvoiceTable, _, _>::new()
        .select::<(
            InvoiceColumns::Id,
            InvoiceColumns::InvoiceNumber,
            InvoiceColumns::ClientId,
            InvoiceColumns::Status,
            InvoiceColumns::TotalCents,
            InvoiceColumns::Currency,
            InvoiceColumns::DueDate,
        )>()
        .from_table()
        .where_clause(
            col(Invoice::company_id())
                .eq(company_id)
                .and(col(Invoice::issue_date()).gt_eq("2024-01-01"))
                .and(col(Invoice::issue_date()).lt_eq("2024-01-31")),
        )
        .order_by(Invoice::issue_date(), false)
        .build();
    print_sql("Invoices for January 2024", &sql);

    // 2. Overdue invoices
    let (sql, _) = Select::<InvoiceTable, _, _>::new()
        .select_all()
        .from_table()
        .where_clause(
            col(Invoice::company_id())
                .eq(company_id)
                .and(col(Invoice::status()).eq("sent"))
                .and(col(Invoice::due_date()).lt(today)),
        )
        .order_by(Invoice::due_date(), true)
        .build();
    print_sql("Overdue invoices", &sql);

    // 3. Draft invoices pending review
    let (sql, _) = Select::<InvoiceTable, _, _>::new()
        .select::<(
            InvoiceColumns::Id,
            InvoiceColumns::InvoiceNumber,
            InvoiceColumns::ClientId,
            InvoiceColumns::TotalCents,
            InvoiceColumns::CreatedAt,
        )>()
        .from_table()
        .where_clause(
            col(Invoice::company_id())
                .eq(company_id)
                .and(col(Invoice::status()).eq("draft")),
        )
        .order_by(Invoice::created_at(), false)
        .build();
    print_sql("Draft invoices pending review", &sql);

    // -------------------------------------------------------------------------
    // CLIENT QUERIES
    // -------------------------------------------------------------------------
    println!("-- Client Queries");
    println!("-- --------------");
    println!();

    // 4. All clients for a company
    let (sql, _) = Select::<ClientTable, _, _>::new()
        .select::<(
            ClientColumns::Id,
            ClientColumns::Name,
            ClientColumns::Email,
            ClientColumns::PreferredCurrency,
        )>()
        .from_table()
        .where_clause(col(Client::company_id()).eq(company_id))
        .order_by(Client::name(), true)
        .build();
    print_sql("All clients for company", &sql);

    // 5. Clients preferring EUR
    let (sql, _) = Select::<ClientTable, _, _>::new()
        .select_all()
        .from_table()
        .where_clause(
            col(Client::company_id())
                .eq(company_id)
                .and(col(Client::preferred_currency()).eq("EUR")),
        )
        .order_by(Client::name(), true)
        .build();
    print_sql("EUR-preferring clients", &sql);

    // 6. Search clients by name
    let (sql, _) = Select::<ClientTable, _, _>::new()
        .select::<(ClientColumns::Id, ClientColumns::Name, ClientColumns::Email)>()
        .from_table()
        .where_clause(
            col(Client::company_id())
                .eq(company_id)
                .and(col(Client::name()).like("%acme%")),
        )
        .limit(10)
        .build();
    print_sql("Search clients by name", &sql);

    // -------------------------------------------------------------------------
    // INVOICE QUERIES
    // -------------------------------------------------------------------------
    println!("-- Invoice Queries");
    println!("-- ---------------");
    println!();

    // 7. All invoices for a specific client
    let client_id = 42_i64;
    let (sql, _) = Select::<InvoiceTable, _, _>::new()
        .select_all()
        .from_table()
        .where_clause(
            col(Invoice::company_id())
                .eq(company_id)
                .and(col(Invoice::client_id()).eq(client_id)),
        )
        .order_by(Invoice::issue_date(), false)
        .build();
    print_sql(&format!("All invoices for client {}", client_id), &sql);

    // 8. Unpaid invoices (sent or overdue)
    let (sql, _) = Select::<InvoiceTable, _, _>::new()
        .select::<(
            InvoiceColumns::Id,
            InvoiceColumns::InvoiceNumber,
            InvoiceColumns::Status,
            InvoiceColumns::TotalCents,
            InvoiceColumns::DueDate,
        )>()
        .from_table()
        .where_clause(
            col(Invoice::company_id())
                .eq(company_id)
                .and(col(Invoice::status()).in_list(vec!["sent", "overdue"])),
        )
        .order_by(Invoice::due_date(), true)
        .build();
    print_sql("Unpaid invoices (sent or overdue)", &sql);

    // 9. Large EUR invoices (> 10,000 EUR)
    let min_amount = 1000000_i64;
    let (sql, _) = Select::<InvoiceTable, _, _>::new()
        .select::<(
            InvoiceColumns::Id,
            InvoiceColumns::InvoiceNumber,
            InvoiceColumns::ClientId,
            InvoiceColumns::TotalCents,
        )>()
        .from_table()
        .where_clause(
            col(Invoice::company_id())
                .eq(company_id)
                .and(col(Invoice::total_cents()).gt(min_amount))
                .and(col(Invoice::currency()).eq("EUR")),
        )
        .order_by(Invoice::total_cents(), false)
        .build();
    print_sql("Large EUR invoices (> 10,000 EUR)", &sql);

    // 10. Recently paid invoices
    let (sql, _) = Select::<InvoiceTable, _, _>::new()
        .select::<(
            InvoiceColumns::Id,
            InvoiceColumns::InvoiceNumber,
            InvoiceColumns::TotalCents,
            InvoiceColumns::PaidAt,
        )>()
        .from_table()
        .where_clause(
            col(Invoice::company_id())
                .eq(company_id)
                .and(col(Invoice::status()).eq("paid"))
                .and(col(Invoice::paid_at()).gt_eq("2024-01-08")),
        )
        .order_by(Invoice::paid_at(), false)
        .build();
    print_sql("Recently paid invoices (last 7 days)", &sql);

    // 11. Foreign currency invoices
    let (sql, _) = Select::<InvoiceTable, _, _>::new()
        .select::<(
            InvoiceColumns::Id,
            InvoiceColumns::InvoiceNumber,
            InvoiceColumns::Currency,
            InvoiceColumns::TotalCents,
        )>()
        .from_table()
        .where_clause(
            col(Invoice::company_id())
                .eq(company_id)
                .and(col(Invoice::currency()).not_eq("EUR")),
        )
        .order_by(Invoice::currency(), true)
        .build();
    print_sql("Foreign currency invoices (not EUR)", &sql);

    // -------------------------------------------------------------------------
    // INVOICE LINE QUERIES
    // -------------------------------------------------------------------------
    println!("-- Invoice Line Queries");
    println!("-- --------------------");
    println!();

    // 12. Invoice lines for a specific invoice
    let invoice_id = 123_i64;
    let (sql, _) = Select::<InvoiceLineTable, _, _>::new()
        .select_all()
        .from_table()
        .where_clause(col(InvoiceLine::invoice_id()).eq(invoice_id))
        .order_by(InvoiceLine::sort_order(), true)
        .build();
    print_sql(&format!("Invoice lines for invoice {}", invoice_id), &sql);

    // 13. High-value line items
    let (sql, _) = Select::<InvoiceLineTable, _, _>::new()
        .select::<(
            InvoiceLineColumns::Id,
            InvoiceLineColumns::InvoiceId,
            InvoiceLineColumns::Description,
            InvoiceLineColumns::TotalCents,
        )>()
        .from_table()
        .where_clause(col(InvoiceLine::total_cents()).gt(100000_i64))
        .order_by(InvoiceLine::total_cents(), false)
        .limit(50)
        .build();
    print_sql("High-value line items (> 1,000)", &sql);

    // -------------------------------------------------------------------------
    // PAYMENT QUERIES
    // -------------------------------------------------------------------------
    println!("-- Payment Queries");
    println!("-- ---------------");
    println!();

    // 14. All payments for an invoice
    let (sql, _) = Select::<PaymentTable, _, _>::new()
        .select_all()
        .from_table()
        .where_clause(col(Payment::invoice_id()).eq(invoice_id))
        .order_by(Payment::paid_at(), false)
        .build();
    print_sql(&format!("Payments for invoice {}", invoice_id), &sql);

    // 15. Recent bank/card payments
    let (sql, _) = Select::<PaymentTable, _, _>::new()
        .select::<(
            PaymentColumns::Id,
            PaymentColumns::InvoiceId,
            PaymentColumns::AmountCents,
            PaymentColumns::PaymentMethod,
            PaymentColumns::PaidAt,
        )>()
        .from_table()
        .where_clause(
            col(Payment::payment_method())
                .in_list(vec!["bank_transfer", "credit_card"])
                .and(col(Payment::paid_at()).gt_eq("2024-01-08")),
        )
        .order_by(Payment::paid_at(), false)
        .limit(100)
        .build();
    print_sql("Recent bank/card payments", &sql);

    // -------------------------------------------------------------------------
    // EXCHANGE RATE QUERIES
    // -------------------------------------------------------------------------
    println!("-- Exchange Rate Queries");
    println!("-- ---------------------");
    println!();

    // 16. Latest USD -> EUR rate
    let (sql, _) = Select::<ExchangeRateTable, _, _>::new()
        .select::<(
            ExchangeRateColumns::Rate,
            ExchangeRateColumns::EffectiveDate,
        )>()
        .from_table()
        .where_clause(
            col(ExchangeRate::from_currency())
                .eq("USD")
                .and(col(ExchangeRate::to_currency()).eq("EUR")),
        )
        .order_by(ExchangeRate::effective_date(), false)
        .limit(1)
        .build();
    print_sql("Latest USD -> EUR exchange rate", &sql);

    // 17. Historical EUR -> GBP rates
    let (sql, _) = Select::<ExchangeRateTable, _, _>::new()
        .select_all()
        .from_table()
        .where_clause(
            col(ExchangeRate::from_currency())
                .eq("EUR")
                .and(col(ExchangeRate::to_currency()).eq("GBP"))
                .and(col(ExchangeRate::effective_date()).gt_eq("2024-01-01"))
                .and(col(ExchangeRate::effective_date()).lt_eq("2024-01-31")),
        )
        .order_by(ExchangeRate::effective_date(), true)
        .build();
    print_sql("EUR -> GBP rates for January 2024", &sql);

    // -------------------------------------------------------------------------
    // PAGINATION EXAMPLE
    // -------------------------------------------------------------------------
    println!("-- Pagination Example");
    println!("-- ------------------");
    println!();

    let page = 3_i64;
    let per_page = 25_i64;
    let offset = (page - 1) * per_page;

    let (sql, _) = Select::<InvoiceTable, _, _>::new()
        .select_all()
        .from_table()
        .where_clause(col(Invoice::company_id()).eq(company_id))
        .order_by(Invoice::created_at(), false)
        .limit(per_page)
        .offset(offset)
        .build();
    print_sql(&format!("Invoices page {} (25 per page)", page), &sql);

    // -------------------------------------------------------------------------
    // COMPANY QUERIES
    // -------------------------------------------------------------------------
    println!("-- Company Queries");
    println!("-- ---------------");
    println!();

    // 18. Get company by ID
    let (sql, _) = Select::<CompanyTable, _, _>::new()
        .select_all()
        .from_table()
        .where_clause(col(Company::id()).eq(company_id))
        .build();
    print_sql("Get company by ID", &sql);

    // 19. Companies using USD
    let (sql, _) = Select::<CompanyTable, _, _>::new()
        .select::<(
            CompanyColumns::Id,
            CompanyColumns::Name,
            CompanyColumns::DefaultCurrency,
        )>()
        .from_table()
        .where_clause(col(Company::default_currency()).eq("USD"))
        .order_by(Company::name(), true)
        .build();
    print_sql("Companies using USD as default currency", &sql);

    println!("-- =============================================================================");
    println!("-- END OF QUERIES");
    println!("-- =============================================================================");
}

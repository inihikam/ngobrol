use actix_web::HttpResponse;
use serde::Serialize;

/// Simple success response (200 OK)
/// Returns data directly without wrapper
pub fn success_response<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(data)
}

/// Created response (201 Created)
/// Returns created resource directly
pub fn created_response<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Created().json(data)
}

/// No content response (204 No Content)
/// Empty response body
pub fn no_content_response() -> HttpResponse {
    HttpResponse::NoContent().finish()
}

/// Paginated response structure
#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total_items: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationMeta {
    pub fn new(page: u32, per_page: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (per_page as f64)).ceil() as u32;

        Self {
            page,
            per_page,
            total_items,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

/// Helper to create paginated response
pub fn paginated_response<T: Serialize>(
    items: Vec<T>,
    page: u32,
    per_page: u32,
    total_items: u64,
) -> HttpResponse {
    let response = PaginatedResponse {
        items,
        pagination: PaginationMeta::new(page, per_page, total_items),
    };

    HttpResponse::Ok().json(response)
}

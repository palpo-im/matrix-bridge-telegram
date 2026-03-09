use salvo::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global metrics counters.
static TELEGRAM_MESSAGES: AtomicU64 = AtomicU64::new(0);
static MATRIX_MESSAGES: AtomicU64 = AtomicU64::new(0);
static MEDIA_TRANSFERS: AtomicU64 = AtomicU64::new(0);
static ERRORS: AtomicU64 = AtomicU64::new(0);

pub fn increment_telegram_messages() {
    TELEGRAM_MESSAGES.fetch_add(1, Ordering::Relaxed);
}

pub fn increment_matrix_messages() {
    MATRIX_MESSAGES.fetch_add(1, Ordering::Relaxed);
}

pub fn increment_media_transfers() {
    MEDIA_TRANSFERS.fetch_add(1, Ordering::Relaxed);
}

pub fn increment_errors() {
    ERRORS.fetch_add(1, Ordering::Relaxed);
}

/// Prometheus-compatible metrics endpoint.
#[handler]
pub async fn metrics_endpoint(res: &mut Response) {
    let tg = TELEGRAM_MESSAGES.load(Ordering::Relaxed);
    let mx = MATRIX_MESSAGES.load(Ordering::Relaxed);
    let media = MEDIA_TRANSFERS.load(Ordering::Relaxed);
    let errs = ERRORS.load(Ordering::Relaxed);

    let output = format!(
        "# HELP bridge_telegram_messages_total Total messages received from Telegram\n\
         # TYPE bridge_telegram_messages_total counter\n\
         bridge_telegram_messages_total {}\n\
         \n\
         # HELP bridge_matrix_messages_total Total messages received from Matrix\n\
         # TYPE bridge_matrix_messages_total counter\n\
         bridge_matrix_messages_total {}\n\
         \n\
         # HELP bridge_media_transfers_total Total media files transferred\n\
         # TYPE bridge_media_transfers_total counter\n\
         bridge_media_transfers_total {}\n\
         \n\
         # HELP bridge_errors_total Total errors encountered\n\
         # TYPE bridge_errors_total counter\n\
         bridge_errors_total {}\n",
        tg, mx, media, errs
    );

    res.render(Text::Plain(output));
}

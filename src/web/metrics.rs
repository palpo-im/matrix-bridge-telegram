use salvo::prelude::*;

#[handler]
pub async fn metrics_endpoint(res: &mut Response) {
    res.render(Text::Plain("# TYPE bridge_messages_total counter\nbridge_messages_total 0\n"));
}

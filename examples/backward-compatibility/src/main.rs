use std::io::{BufRead, BufReader, Read};
use opentelemetry::{
    global,
    sdk::{propagation::TraceContextPropagator, trace as sdktrace, Resource},
    trace::{Span, TraceContextExt, TraceError, Tracer, SpanId, TraceId},
    Context, Key, KeyValue
};
use opentelemetry_otlp::{WithExportConfig};
use opentelemetry_semantic_conventions as semcov;

use reqwest;
use hyper::Client;

async fn outgoing_request(
    c_span_id: SpanId, p_span_id: SpanId, trace_id: TraceId
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {

    let client_2 = reqwest::Client::builder().http1_title_case_headers()
        .build()?;

    let b3_header = format!("{0}-{1}-{2}-{3}", trace_id,c_span_id, "0",p_span_id);

    let _res = client_2
        .get("http://localhost:8000/api/test")
        .header("B3", b3_header)
        .send()
        .await?
        .text()
        .await?;

    println!("{}", "API Called using b3 single header");
    Ok(())
}

async fn outgoing_request_multi(
    c_span_id: SpanId, p_span_id: SpanId, trace_id: TraceId
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let client = Client::new();
    
    let req = hyper::Request::builder()
        .method(hyper::Method::GET)
        .uri("http://localhost:8000/api/test")
        .header("X-B3-TraceId", trace_id.to_string())
        .header("X-B3-SpanId", c_span_id.to_string())
        .header("X-B3-ParentSpanId", p_span_id.to_string())
        .body(hyper::Body::empty()).unwrap();

    let resp = client.request(req).await?;
    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
    let _body = String::from_utf8(body_bytes.to_vec()).unwrap();

    println!("{}", "API Called using b3 multi header");
    Ok(())
}

fn read_dt_metadata() -> Resource {
    fn read_single(path: &str, metadata: &mut Vec<KeyValue>) -> std::io::Result<()> {
        let mut file = std::fs::File::open(path)?;
        if path.starts_with("dt_metadata") {
            let mut name = String::new();
            file.read_to_string(&mut name)?;

            file = std::fs::File::open(name)?;
        }

        for line in BufReader::new(file).lines() {
            if let Some((k, v)) = line?.split_once('=') {
                metadata.push(KeyValue::new(k.to_string(), v.to_string()))
            }
        }

        Ok(())
    }

    let mut metadata = Vec::new();
    for name in [
        "metadata 1",
        "metadata 2",
    ] {
        let _ = read_single(name, &mut metadata);
    }

    Resource::new(metadata)
}


fn init_tracer() -> Result<sdktrace::Tracer, TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let mut resource = Resource::new(vec![
        semcov::resource::SERVICE_NAME.string("fdse-1730-service-rust"),
        semcov::resource::DEPLOYMENT_ENVIRONMENT.string("fdse-1730-env"),
        semcov::resource::SERVICE_VERSION.string("1.0.1"), 
    ]);

    resource = resource.merge(&read_dt_metadata());

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            sdktrace::config()
                .with_resource(resource)
                .with_sampler(sdktrace::Sampler::AlwaysOn)
                .with_backward_compatible(true)
        )
        .install_simple()
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tracer = init_tracer()?;
    let m_span = tracer.start("Main Span");
    let main_cx = Context::current_with_span(m_span);
    let mut span = tracer.start_with_context("operation waiting", &main_cx);
    span.set_attribute(KeyValue::new("sample_key", "sample_value")); 
    span.add_event(
        "Waited 5 sec before call!".to_string(),
        vec![Key::new("bogons").i64(100)],
    );

    println!("Tracer created with ID: {}", span.span_context().trace_id());

    let p_span_id = span.span_context().span_id();
    let parent_cx = Context::current_with_span(span);

    let mut c_span = tracer.start_with_context("API Call-b3single", &parent_cx);
    let _ = outgoing_request(c_span.span_context().span_id(), p_span_id, c_span.span_context().trace_id()).await;
    c_span.set_attribute(KeyValue::new("header-type", "b3 single"));
    c_span.add_event(
        "made an API call to laravel".to_string(),
        vec![Key::new("bogons").i64(100)],
    );
    c_span.end();

    let mut a_span = tracer.start_with_context("API Call- b3multi", &parent_cx);
    let _ = outgoing_request_multi(a_span.span_context().span_id(), p_span_id, a_span.span_context().trace_id()).await;
    a_span.set_attribute(KeyValue::new("header-type", "b3 multi"));
    a_span.add_event(
        "made an API call to laravel".to_string(),
        vec![Key::new("bogons").i64(100)],
    );
    a_span.end();

    drop(parent_cx);
    drop(main_cx);
    global::shutdown_tracer_provider();
    Ok(())
}

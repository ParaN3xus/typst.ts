use std::fs;

use clap::Parser;
use reflexo_typst::{
    args::CompileOnceArgs,
    vector::{
        incr::{IncrDocClient, IncrDocServer},
        stream::BytesModuleStream,
    },
    TypstDocument, TypstPagedDocument,
};
use reflexo_vec2svg::{
    ir::{Abs, Point, Rect},
    IncrSvgDocClient,
};

// cargo run -p typst-ts-incr-test

fn main() {
    let args = CompileOnceArgs::parse_from(["tinymist", "main.typ"]);
    let verse = args
        .resolve_system()
        .expect("failed to resolve system universe");

    let world = verse.snapshot();
    let doc: TypstPagedDocument = typst::compile(&world).output.unwrap();

    let mut incr_server = IncrDocServer::default();
    let mut incr_client = IncrDocClient::default();
    let mut incr_svg_client = IncrSvgDocClient::default();

    let window = Rect {
        lo: Point::new(Abs::from(0.), Abs::from(0.)),
        hi: Point::new(Abs::from(1e33), Abs::from(1e33)),
    };

    let _ = incr_server.pack_current();

    let server_delta = incr_server.pack_delta(&TypstDocument::Paged(doc.into()));
    let server_delta = BytesModuleStream::from_slice(&server_delta).checkout_owned();
    incr_client.merge_delta(server_delta);

    let svg_text = incr_svg_client.render_in_window(&mut incr_client, window);

    fs::write("output.svg", &svg_text).expect("Unable to write file");
}

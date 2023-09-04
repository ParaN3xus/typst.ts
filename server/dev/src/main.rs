use clap::Parser;
use log::info;
use std::{path::PathBuf, process::exit, sync::Mutex};

use typst_ts_compiler::service::{CompileExporter, Compiler, DiagObserver, WrappedCompiler};
use typst_ts_core::path::PathClean;
use typst_ts_dev_server::{http::run_http, utils::async_continue, CompileOpts, RunSubCommands};

use typst_ts_dev_server::{CompileCorpusArgs, CompileSubCommands, Opts, Subcommands};

static COMPILER_PATH: Mutex<Option<String>> = Mutex::new(None);

fn main() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .filter_module("typst::", log::LevelFilter::Warn)
        .filter_module("typst_library::", log::LevelFilter::Warn)
        .try_init();

    let opts = Opts::parse();

    match opts.sub {
        Subcommands::Compile(compile_opts) => {
            find_compiler_path(&compile_opts);
            match compile_opts.sub {
                CompileSubCommands::Corpus(args) => compile_corpus(args),
            }
        }
        Subcommands::Run(run_sub) => match run_sub {
            RunSubCommands::Http(args) => async_continue(async move {
                run_http(args).await;
                exit(0);
            }),
        },
    };

    #[allow(unreachable_code)]
    {
        unreachable!("The subcommand must exit the process.");
    }
}

fn find_program_path(dir: &str, program: &str) -> Option<String> {
    let program = PathBuf::from(dir).join(program);
    if program.exists() {
        return Some(program.to_str().unwrap().to_string());
    } else if program.with_extension("exe").exists() {
        return Some(program.with_extension("exe").to_str().unwrap().to_string());
    }
    None
}

fn find_compiler_path(compile_opts: &CompileOpts) {
    const COMPILER_NAME: &str = "typst-ts-cli";

    let mut compiler_path = COMPILER_PATH.lock().unwrap();

    if !compile_opts.compiler.is_empty() {
        let compiler = compile_opts.compiler.clone();
        match compiler.as_str() {
            "debug" => {
                *compiler_path = find_program_path("target/debug", COMPILER_NAME);
            }
            "release" => {
                *compiler_path = find_program_path("target/release", COMPILER_NAME);
            }
            _ => {
                let path = PathBuf::from(&compiler);
                *compiler_path = find_program_path(
                    path.parent().unwrap().to_str().unwrap(),
                    path.file_name().unwrap().to_str().unwrap(),
                );
            }
        }
    } else {
        if compiler_path.is_none() {
            *compiler_path = find_program_path(".", COMPILER_NAME);
        }

        if compiler_path.is_none() {
            *compiler_path = find_program_path("target/debug", COMPILER_NAME);
        }

        if compiler_path.is_none() {
            *compiler_path = find_program_path("target/release", COMPILER_NAME);
        }
    }

    if compiler_path.is_none() {
        eprintln!(
            "Cannot find typst-ts-cli in current directory, target/debug, or target/release."
        );
        exit(1);
    }
    info!("using compiler path: {}", compiler_path.clone().unwrap());
}

fn compile_corpus(args: CompileCorpusArgs) {
    let corpus_path = "fuzzers/corpora";

    let mut compile_formats = args.format.clone();
    if compile_formats.is_empty() {
        compile_formats.push("svg".to_owned());
        compile_formats.push("sir".to_owned());
    }

    let compile_args = typst_ts_cli::CompileArgs {
        compile: typst_ts_cli::CompileOnceArgs {
            font: typst_ts_cli::FontArgs { paths: vec![] },
            workspace: ".".to_owned(),
            entry: "".to_owned(),
            ..Default::default()
        },
        format: compile_formats.clone(),
        ..Default::default()
    };

    let driver = typst_ts_cli::compile::create_driver(compile_args.compile.clone());

    let mut driver = CompileExporter::new(driver);

    let mut compile = |cat: String, name: String| {
        let entry = PathBuf::from(corpus_path).join(cat).join(name).clean();

        let exporter = typst_ts_cli::export::prepare_exporters(&compile_args, &entry);
        driver.set_exporter(exporter);
        driver.inner_mut().set_entry_file(entry);

        driver.with_compile_diag::<true, _>(|driver| driver.compile());

        // if status.code().unwrap() != 0 {
        //     eprintln!("compile corpus failed.");
        //     exit(status.code().unwrap());
        // }
    };

    // get all corpus in workspace_path

    for cat in args.catergories.clone() {
        info!("compile corpus in {cat}...");

        let cat_dir = PathBuf::from(corpus_path).join(&cat);

        let corpora = std::fs::read_dir(&cat_dir).unwrap();

        for corpus in corpora {
            let corpus_name = corpus.unwrap().file_name();
            if !corpus_name.to_string_lossy().ends_with(".typ") {
                continue;
            }
            info!("compile corpus: {cat:10} {}", corpus_name.to_string_lossy());

            compile(cat.clone(), corpus_name.to_string_lossy().to_string());
        }
    }
    exit(0);
}

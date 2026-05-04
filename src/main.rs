use clap::{Parser, ValueEnum};
use syscall_compat_tests::{
    reporter::{print_console_report, write_json_report, write_markdown_report},
    runner::TestRunner,
    testcases::register_all,
};

#[derive(Parser)]
#[command(
    name = "sct-run",
    about = "syscall-compat-tests: Linux syscall compatibility test suite for rCore-OS ecosystem",
    version
)]
struct Cli {
    #[arg(short, long, default_value = "linux")]
    target: String,
    #[arg(short, long, value_enum, default_value = "console")]
    format: OutputFormat,
    #[arg(short, long)]
    output: Option<String>,
    #[arg(short, long, value_enum)]
    category: Option<Category>,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat { Console, Json, Markdown, All }

#[derive(Clone, ValueEnum)]
enum Category { FileIo, Process, Memory, Time }

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut runner = TestRunner::new();
    register_all(&mut runner);

    if let Some(cat) = &cli.category {
        use syscall_compat_tests::runner::SyscallCategory;
        let sc = match cat {
            Category::FileIo => SyscallCategory::FileIO,
            Category::Process => SyscallCategory::Process,
            Category::Memory => SyscallCategory::Memory,
            Category::Time => SyscallCategory::Time,
        };
        runner.filter_category(sc);
    }

    println!("Running {} test cases against target: {}", runner.test_count(), cli.target);
    let results = runner.run_all();
    let out_path = cli.output.as_deref().unwrap_or("report");

    match cli.format {
        OutputFormat::Console => print_console_report(&results, &cli.target),
        OutputFormat::Json => write_json_report(&results, &cli.target, &format!("{}.json", out_path))?,
        OutputFormat::Markdown => write_markdown_report(&results, &cli.target, &format!("{}.md", out_path))?,
        OutputFormat::All => {
            print_console_report(&results, &cli.target);
            write_json_report(&results, &cli.target, &format!("{}.json", out_path))?;
            write_markdown_report(&results, &cli.target, &format!("{}.md", out_path))?;
        }
    }
    Ok(())
}

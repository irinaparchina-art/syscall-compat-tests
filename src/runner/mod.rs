use serde::{Deserialize, Serialize};

/// The result of running a single syscall test case.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestStatus {
    /// Syscall behaved as expected (return value and errno matched).
    Pass,
    /// Return value or errno did not match expected values.
    Fail {
        expected_ret: i64,
        actual_ret: i64,
        expected_errno: Option<i32>,
        actual_errno: Option<i32>,
    },
    /// Syscall is not implemented (returned ENOSYS).
    Unimplemented,
    /// The test itself encountered an internal error.
    Error(String),
}

/// A single test case result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub name: String,
    pub syscall: String,
    pub category: SyscallCategory,
    pub status: TestStatus,
    pub description: String,
    pub duration_us: u64,
}

/// Categorizes syscalls for reporting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SyscallCategory {
    FileIO,
    Process,
    Memory,
    Signal,
    Network,
    Time,
    Other,
}

impl std::fmt::Display for SyscallCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SyscallCategory::FileIO => "File I/O",
            SyscallCategory::Process => "Process",
            SyscallCategory::Memory => "Memory",
            SyscallCategory::Signal => "Signal",
            SyscallCategory::Network => "Network",
            SyscallCategory::Time => "Time",
            SyscallCategory::Other => "Other",
        };
        write!(f, "{}", s)
    }
}

/// Trait that every test case must implement.
pub trait SyscallTest: Send + Sync {
    /// Short unique identifier, e.g. "file_open_basic"
    fn name(&self) -> &str;

    /// The primary syscall being tested, e.g. "open"
    fn syscall(&self) -> &str;

    /// Category for grouping in reports.
    fn category(&self) -> SyscallCategory;

    /// Human-readable description of what this test verifies.
    fn description(&self) -> &str;

    /// Execute the test and return a result.
    fn run(&self) -> TestResult;
}

/// Collects and runs all registered test cases.
pub struct TestRunner {
    tests: Vec<Box<dyn SyscallTest>>,
    filter: Option<SyscallCategory>,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            filter: None,
        }
    }

    /// Register a test case.
    pub fn register(&mut self, test: Box<dyn SyscallTest>) {
        self.tests.push(test);
    }

    /// Only run tests in a specific category.
    pub fn filter_category(&mut self, category: SyscallCategory) {
        self.filter = Some(category);
    }

    /// Run all registered (or filtered) tests and return results.
    pub fn run_all(&self) -> Vec<TestResult> {
        self.tests
            .iter()
            .filter(|t| {
                self.filter
                    .as_ref()
                    .map(|f| &t.category() == f)
                    .unwrap_or(true)
            })
            .map(|t| t.run())
            .collect()
    }

    pub fn test_count(&self) -> usize {
        self.tests.len()
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

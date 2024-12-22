use anathema_backend::tui::TuiBackend;
use anathema_runtime::Runtime;
use testcase::TestCase;

mod error;
mod parse;
mod testcase;

type TestRuntime = Runtime<TuiBackend, ()>;

pub struct TestRunner<'src> {
    cases: Vec<TestCase<'src>>,
    runtime: TestRuntime,
}

impl<'src> TestRunner<'src> {
    pub fn new(cases: Vec<TestCase<'src>>, runtime: TestRuntime) -> Self {
        Self { cases, runtime }
    }

    pub fn run(self) {
        for case in self.cases {
            match case.run(&self.runtime) {
                _ => panic!()
            }
            println!("running {}", case.title());
        }
    }
}

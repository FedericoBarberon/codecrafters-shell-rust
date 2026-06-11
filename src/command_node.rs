use crate::executor::Executable;

#[derive(Debug, PartialEq)]
pub enum CommandNode<C> {
    Single(C),
}

impl<C: Executable> Executable for CommandNode<C> {
    fn execute(
        &self,
        in_buf: &mut impl std::io::prelude::Read,
        out_buf: &mut impl std::io::prelude::Write,
        err_buf: &mut impl std::io::prelude::Write,
    ) -> Result<crate::executor::ExecutionResult, crate::executor::ExecutionError> {
        match self {
            Self::Single(c) => c.execute(in_buf, out_buf, err_buf),
        }
    }
}

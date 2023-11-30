use super::{effectfull_context::EffectfullContext, test_context::TestContext};

pub trait EffectfullRequest<Context> {
    type Result;
    fn execute(self, context: &mut Context) -> Self::Result;
}

pub trait Context: Sized {
    fn apply<T: EffectfullRequest<Self>>(
        &mut self,
        arg: T,
    ) -> <T as EffectfullRequest<Self>>::Result {
        arg.execute(self)
    }
}

#[derive(Clone)]
pub enum GeneralContext {
    Test(TestContext),
    Effectfull(EffectfullContext),
}

impl Context for GeneralContext {}

impl<T, R> EffectfullRequest<GeneralContext> for T
where
    T: EffectfullRequest<TestContext, Result = R>
        + EffectfullRequest<EffectfullContext, Result = R>,
{
    type Result = R;
    fn execute(self, context: &mut GeneralContext) -> Self::Result {
        match context {
            GeneralContext::Test(context) => self.execute(context),
            GeneralContext::Effectfull(context) => self.execute(context),
        }
    }
}

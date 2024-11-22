pub trait Runnable: Send {
    fn run(self: Box<Self>);
}

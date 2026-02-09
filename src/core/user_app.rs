pub trait UserApp {
    fn init() -> Self;

    fn update();
    fn render();
}
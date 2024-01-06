pub trait ZScreen {
    fn newline(&self);
    fn print(&self, str: String);
    fn read(&self) -> char;
    fn readline(&self) -> String;
    fn exit(&self);
    fn random(&self, limit: u16) -> u16;
    fn set_status(&self, status: String);
    fn get_width(&self);
    fn get_height(&self);
    fn restart(&self);
    fn save(&self, state: Vec<u8>);
    fn restore(&self) -> Vec<u8>;
    fn set_window(&self, num: u16);
    fn split_window(&self, height: u16);
    fn erase_window(&self, num: u16);
    fn move_cursor(&self, x: u8, y: u8);
    fn print_number(&self, num: u16);
    fn print_char(&self, char: char);
}

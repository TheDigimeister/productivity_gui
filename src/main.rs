use iced::{Command, Element, Settings, Theme, executor, widget::{column, row, text, TextInput, Button, Container, Scrollable}, Length};
use iced::Application;

fn main() -> iced::Result {
    ProductivityApp::run(Settings::default())
}

#[derive(Debug, Clone)]
enum Message {
    TodoInputChanged(String),
    AddTodo,
    CalendarDateSelected(chrono::NaiveDate),
}

struct ProductivityApp {
    todo_input: String,
    todos: Vec<String>,
    selected_date: Option<chrono::NaiveDate>,
}

impl Application for ProductivityApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            ProductivityApp {
                todo_input: String::new(),
                todos: Vec::new(),
                selected_date: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Productivity GUI - To-Do & Calendar")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::TodoInputChanged(input) => {
                self.todo_input = input;
            }
            Message::AddTodo => {
                if !self.todo_input.trim().is_empty() {
                    self.todos.push(self.todo_input.trim().to_string());
                    self.todo_input.clear();
                }
            }
            Message::CalendarDateSelected(date) => {
                self.selected_date = Some(date);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let todo_list = self.todos.iter().fold(
            column![text("To-Do List:")],
            |col, todo| col.push(text(todo)),
        );

        let todo_input = TextInput::new("Add a to-do...", &self.todo_input)
            .on_input(Message::TodoInputChanged)
            .on_submit(Message::AddTodo);
        let add_button = Button::new(text("Add")).on_press(Message::AddTodo);
        // Placeholder for calendar widget, since iced does not provide a Calendar widget by default.
        let calendar_placeholder = text("Calendar widget not implemented");

        let selected_date_text = if let Some(date) = self.selected_date {
            text(format!("Selected date: {}", date))
        } else {
            text("No date selected")
        };

        row![
            column![todo_list, row![todo_input, add_button]].width(Length::FillPortion(1)),
            column![text("Calendar:"), calendar_placeholder, selected_date_text].width(Length::FillPortion(1)),
        ]
        .spacing(20)
        .padding(20)
        .into()
    }
}

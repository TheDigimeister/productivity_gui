use iced::{Command, Element, Settings, Theme, executor, widget::{column, row, text, TextInput, Button, Container, Scrollable, PickList}, Length};
use iced::Application;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::fmt;

const TODO_CSV_PATH: &str = "todos.csv";

fn main() -> iced::Result {
    ProductivityApp::run(Settings::default())
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TodoItem {
    description: String,
    completed: bool,
    category: String,
}

#[derive(Debug, Clone)]
enum Message {
    TodoInputChanged(String),
    CategoryInputChanged(String),
    AddTodo,
    ToggleTodoCompleted(usize),
    ToggleShowCompleted,
    SortByCategory,
    FilterCategoryChanged(FilterCategory),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FilterCategory(pub Option<String>);

impl fmt::Display for FilterCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(cat) => write!(f, "{}", cat),
            None => write!(f, "All"),
        }
    }
}

struct ProductivityApp {
    todo_input: String,
    category_input: String,
    todos: Vec<TodoItem>,
    show_completed: bool,
    sort_by_category: bool,
    filter_category: Option<String>,
}

impl ProductivityApp {
    fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self.todos.iter().map(|t| t.category.clone()).collect();
        if !self.category_input.trim().is_empty() && !cats.contains(&self.category_input) {
            cats.push(self.category_input.clone());
        }
        cats.sort();
        cats.dedup();
        cats
    }
    fn filter_categories(&self) -> Vec<FilterCategory> {
        let mut cats: Vec<FilterCategory> = self.categories().into_iter().map(|c| FilterCategory(Some(c))).collect();
        cats.insert(0, FilterCategory(None)); // None means show all
        cats
    }
}

impl Application for ProductivityApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut todos = Vec::new();
        if let Ok(file) = File::open(TODO_CSV_PATH) {
            let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(BufReader::new(file));
            for result in rdr.deserialize() {
                if let Ok(todo) = result {
                    todos.push(todo);
                }
            }
        }
        (
            ProductivityApp {
                todo_input: String::new(),
                category_input: String::new(),
                todos,
                show_completed: false,
                sort_by_category: false,
                filter_category: None,
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
            Message::CategoryInputChanged(input) => {
                self.category_input = input;
            }
            Message::AddTodo => {
                if !self.todo_input.trim().is_empty() {
                    self.todos.push(TodoItem {
                        description: self.todo_input.trim().to_string(),
                        completed: false,
                        category: self.category_input.trim().to_string(),
                    });
                    self.todo_input.clear();
                    self.category_input.clear();
                    save_todos(&self.todos);
                }
            }
            Message::ToggleTodoCompleted(idx) => {
                if let Some(todo) = self.todos.get_mut(idx) {
                    todo.completed = !todo.completed;
                    save_todos(&self.todos);
                }
            }
            Message::ToggleShowCompleted => {
                self.show_completed = !self.show_completed;
            }
            Message::SortByCategory => {
                self.sort_by_category = !self.sort_by_category;
                if self.sort_by_category {
                    self.todos.sort_by(|a, b| a.category.cmp(&b.category));
                }
            }
            Message::FilterCategoryChanged(cat) => {
                self.filter_category = cat.0;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let todos_iter = self.todos.iter().enumerate()
            .filter(|(_, todo)| self.show_completed || !todo.completed)
            .filter(|(_, todo)| {
                if let Some(ref cat) = self.filter_category {
                    &todo.category == cat
                } else {
                    true
                }
            });
        let todo_list = todos_iter.fold(
            column![text("To-Do List:")],
            |col, (idx, todo)| {
                let check = if todo.completed { "[x]" } else { "[ ]" };
                col.push(
                    row![
                        Button::new(text(check)).on_press(Message::ToggleTodoCompleted(idx)),
                        text(&todo.description),
                        text(format!("[{}]", todo.category)),
                    ]
                )
            },
        );
        let todo_input = TextInput::new("Add a to-do...", &self.todo_input)
            .on_input(Message::TodoInputChanged)
            .on_submit(Message::AddTodo);
        let categories = self.categories();
        let picklist = PickList::new(
            categories.clone(),
            if self.category_input.is_empty() { None } else { Some(self.category_input.clone()) },
            Message::CategoryInputChanged,
        ).placeholder("Category");
        let filter_picklist = PickList::new(
            self.filter_categories(),
            self.filter_category.clone().map(|c| FilterCategory(Some(c))).or(Some(FilterCategory(None))),
            Message::FilterCategoryChanged,
        ).placeholder("Filter by Category");
        let add_button = Button::new(text("Add")).on_press(Message::AddTodo);
        let show_completed_button = Button::new(
            text(if self.show_completed { "Hide Completed" } else { "Show Completed" })
        ).on_press(Message::ToggleShowCompleted);
        let sort_by_category_button = Button::new(
            text(if self.sort_by_category { "Unsort" } else { "Sort by Category" })
        ).on_press(Message::SortByCategory);
        // Placeholder for calendar widget, since iced does not provide a Calendar widget by default.
        let calendar_placeholder = text("Calendar widget not implemented");

        row![
            column![todo_list, row![todo_input, picklist, add_button, show_completed_button, sort_by_category_button, filter_picklist]].width(Length::FillPortion(1)),
            column![text("Calendar:"), calendar_placeholder].width(Length::FillPortion(1)),
        ]
        .spacing(20)
        .padding(20)
        .into()
    }
}

fn save_todos(todos: &Vec<TodoItem>) {
    if let Ok(file) = OpenOptions::new().write(true).create(true).truncate(true).open(TODO_CSV_PATH) {
        let mut wtr = csv::WriterBuilder::new().has_headers(false).from_writer(BufWriter::new(file));
        for todo in todos {
            let _ = wtr.serialize(todo);
        }
        let _ = wtr.flush();
    }
}

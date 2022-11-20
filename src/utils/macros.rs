/// Creates a `Column` with the given children.
#[macro_export]
// todo: bug in rust plugin - rename to column
macro_rules! columee {
    () => (
        iced::widget::Column::new()
    );
    ($($x:expr),+ $(,)?) => (
        iced::widget::Column::with_children(vec![$(iced::Element::from($x)),+])
    );
}

/// Creates a `Row` with the given children.

#[macro_export]
macro_rules! row {
    () => (
        iced:widget::Row::new()
    );
    ($($x:expr),+ $(,)?) => (
        iced::widget::Row::with_children(vec![$(iced::Element::from($x)),+])
    );
}

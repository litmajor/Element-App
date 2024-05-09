table! {
    income_sources (id) {
        id -> Int4,
        user_id -> Int4,
        name -> Text,
        amount -> Float8,
        date -> Date,
        details -> Nullable<Text>,

    }
}
table! {
    expense_categories (id) {
        id -> Int4,
        name -> Text,
    }
}

table! {
    expense_transactions (id) {
        id -> Int4,
        user_id -> Int4,
        category_id -> Nullable<Int4>,
        amount -> Float8,
        date -> Date,
        description -> Nullable<Text>,
    }
}
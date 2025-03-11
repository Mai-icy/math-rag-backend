// @generated automatically by Diesel CLI.

diesel::table! {
    chats (chat_id) {
        chat_id -> Uuid,
        user_id -> Uuid,
        #[max_length = 100]
        title -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    messages (message_id) {
        message_id -> Uuid,
        chat_id -> Uuid,
        #[max_length = 20]
        role -> Varchar,
        content -> Text,
        timestamp -> Nullable<Timestamp>,
    }
}

diesel::table! {
    user_sessions (session_id) {
        session_id -> Uuid,
        user_id -> Uuid,
        token -> Text,
        created_at -> Nullable<Timestamp>,
        expires_at -> Timestamp,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Uuid,
        #[max_length = 50]
        username -> Varchar,
        #[max_length = 100]
        email -> Varchar,
        password_hash -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(chats -> users (user_id));
diesel::joinable!(messages -> chats (chat_id));
diesel::joinable!(user_sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    chats,
    messages,
    user_sessions,
    users,
);

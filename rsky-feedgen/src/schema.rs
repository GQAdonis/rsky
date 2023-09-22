// @generated automatically by Diesel CLI.

diesel::table! {
    follow (uri) {
        uri -> Varchar,
        cid -> Varchar,
        author -> Varchar,
        subject -> Varchar,
        createdAt -> Varchar,
        indexedAt -> Varchar,
        prev -> Nullable<Varchar>,
        sequence -> Nullable<Int8>,
    }
}

diesel::table! {
    image (cid) {
        cid -> Varchar,
        alt -> Nullable<Varchar>,
        postCid -> Varchar,
        postUri -> Varchar,
        createdAt -> Varchar,
        indexedAt -> Varchar,
        labels -> Nullable<Array<Nullable<Text>>>,
    }
}

diesel::table! {
    like (uri) {
        uri -> Varchar,
        cid -> Varchar,
        author -> Varchar,
        subjectCid -> Varchar,
        subjectUri -> Varchar,
        createdAt -> Varchar,
        indexedAt -> Varchar,
        prev -> Nullable<Varchar>,
        sequence -> Nullable<Int8>,
    }
}

diesel::table! {
    membership (did) {
        did -> Varchar,
        included -> Bool,
        excluded -> Bool,
        list -> Varchar,
    }
}

diesel::table! {
    post (uri) {
        uri -> Varchar,
        cid -> Varchar,
        replyParent -> Nullable<Varchar>,
        replyRoot -> Nullable<Varchar>,
        indexedAt -> Varchar,
        prev -> Nullable<Varchar>,
        sequence -> Nullable<Int8>,
        text -> Nullable<Varchar>,
        lang -> Nullable<Varchar>,
    }
}

diesel::table! {
    sub_state (service) {
        service -> Varchar,
        cursor -> Int8,
    }
}

diesel::table! {
    visitor (id) {
        id -> Int4,
        did -> Varchar,
        web -> Varchar,
        visited_at -> Varchar,
        feed -> Nullable<Varchar>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    follow,
    image,
    like,
    membership,
    post,
    sub_state,
    visitor,
);

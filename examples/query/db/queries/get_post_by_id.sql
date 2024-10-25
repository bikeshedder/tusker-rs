SELECT id,
    author,
    text,
    created,
    deleted
FROM post
WHERE id = $1

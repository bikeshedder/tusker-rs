SELECT id,
    author,
    text,
    created
FROM post
WHERE deleted IS NULL
ORDER BY created DESC
LIMIT $1 OFFSET $2

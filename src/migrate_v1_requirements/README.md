# README

```bash
for file in path/to/requirements/dir/*; do
  migrate_v1_requirements "$file" -o .
done
```
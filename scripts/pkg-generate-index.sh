#!/bin/sh

# Check if bindings directory contains any .ts files
if [ ! -d bindings ] || [ -z "$(ls bindings/*.ts 2>/dev/null)" ]; then
  echo "Error: No TypeScript definition files found in 'bindings' directory."
  exit 1
fi

echo "Generating 'index.d.ts' with exports from TypeScript definitions..."
echo "// Auto-generated index.d.ts" > dist/index.d.ts || { echo "Error: Failed to create 'index.d.ts'"; exit 1; }

for file in bindings/*.ts; do
  filename=$(basename "$file" .ts)
  echo "Exporting module: $filename"
  echo "export * from './$filename';" >> dist/index.d.ts
done

echo "'index.d.ts' generated successfully."


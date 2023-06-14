# Define the first element
first_key='1'
first_val='{"name": "Submarine #1", "description": "A submarine with blue neon lights and with a red background.","attributes": [{"trait_type": "Background","value": "Red"},{"trait_type": "Lights","value": "Cyan"}]}'

# Create the remaining elements using jq and a loop
# data=$(jq --argjson val "$first_val" --arg key "$first_key" '.[$key] += $val')
# data=$(jq -n --argjson val "$first_val" '[range(1; 1000) | . as $i | .["\($i)"] += $val]')

# data=$(jq -n --argjson val "$first_val" '[range(1000) | .+1 as $i | {($i | tostring): $val}] | add')
data=$(jq -n --argjson first_val "$first_val" 'reduce range(1000) as $i ({}; . + {($i+1 | tostring): $first_val})')

echo "$data" | jq '.' > "data.json"

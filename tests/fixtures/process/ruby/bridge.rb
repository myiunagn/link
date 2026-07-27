require 'json'

input = JSON.parse(STDIN.read)
function_name = input['function']
args = input['args'] || []

result = case function_name
when 'add'
  if args.length < 2
    { 'error' => 'add requires 2 arguments' }
  else
    a = args[0].is_a?(Float) ? args[0].to_f : args[0].to_i
    b = args[1].is_a?(Float) ? args[1].to_f : args[1].to_i
    a + b
  end
when 'subtract'
  if args.length < 2
    { 'error' => 'subtract requires 2 arguments' }
  else
    a = args[0].is_a?(Float) ? args[0].to_f : args[0].to_i
    b = args[1].is_a?(Float) ? args[1].to_f : args[1].to_i
    a - b
  end
when 'multiply'
  if args.length < 2
    { 'error' => 'multiply requires 2 arguments' }
  else
    a = args[0].is_a?(Float) ? args[0].to_f : args[0].to_i
    b = args[1].is_a?(Float) ? args[1].to_f : args[1].to_i
    a * b
  end
when 'greet'
  if args.empty?
    { 'error' => 'greet requires 1 argument' }
  else
    "Hello from Ruby, #{args[0]}!"
  end
else
  { 'error' => "Unknown function: #{function_name}" }
end

if result.is_a?(Hash) && result.key?('error')
  puts JSON.generate(result)
else
  puts JSON.generate({ 'result' => result })
end
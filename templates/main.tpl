{{ template "print_env_value_or_secret" . -}}
{{ template "print_env_value_or_secret" . -}}
{{ template "print_env_value_or_secret" . }}
{{$test}}
{{.name}}

{{ abbrev 5 $test}}
{{ abbrev 5 .name}}
{{ template "print_env_value_or_secret" . }}
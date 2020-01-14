{{ .cluster.values.security.passwords_secret_name }}
{{ range .cluster.scripts -}}
{{ indent 4 .name }}: |
{{ indent 8 .script }}
{{ end -}}
{{ .cluster.values.security.pg_password }}
{{ template "print_env_value_or_secret" . }}
{{ $test }}

{{ .cluster.values.on_load.startup }}

{{ define "print_env_value_or_secret"}}
{{- if .cluster.values.security.passwords_secret_name }}
{{- if .cluster.values.security.pg_password }}
- name: {{ .cluster.values.security.passwords_secret_name }}
  valueFrom:
    secretKeyRef:
      name: {{ .cluster.values.security.passwords_secret_name }}
      key: {{ .cluster.values.security.pg_password }}
{{- end }}
{{- end }}
{{ end -}}

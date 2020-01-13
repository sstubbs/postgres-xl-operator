{{- define "print_env_value_or_secret" -}}
{{- if and .cluster.values.security.passwords_secret_name .cluster.values.security.pg_password }}
- name: {{ .cluster.values.security.passwords_secret_name }}
  valueFrom:
    secretKeyRef:
      name: {{ .cluster.values.security.passwords_secret_name }}
      key: {{ .cluster.values.security.pg_password }}
{{- end -}}
{{- end -}}

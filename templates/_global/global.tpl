{{ $app_name := printf "%s-%s" .cleaned_release_name .cluster.cleaned_name }}

{{ define "global_labels" }}
{{ range .cluster.selector_labels -}}
{{ .name | indent 4 }}: {{ .content }}
{{ end -}}
{{ range .cluster.global_labels -}}
{{ .name | indent 4 }}: {{ .content }}
{{ end -}}
{{- if .cluster.values.extra_labels }}
{{ range .cluster.values.extra_labels -}}
{{ .name | indent 4 }}: {{ .content }}
{{ end -}}
{{- end }}
{{- end -}}

{{ define "global_secret" }}
{{ if and .cluster.values.security.passwords_secret_name .cluster.values.security.pg_password }}
          - name: PGPASSWORD
            valueFrom:
              secretKeyRef:
                key: {{ .cluster.values.security.pg_password }}
                name: {{ .cluster.values.security.passwords_secret_name }}
{{- end -}}
{{- end -}}


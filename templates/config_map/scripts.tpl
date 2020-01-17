apiVersion: v1
kind: ConfigMap
metadata:
  name: {{ $app_name }}-scripts
  labels:
    app.kubernetes.io/component: scripts
{{- template "global_labels" . }}
data:
  # Load scripts from the scripts directory.
{{ range .cluster.scripts -}}
{{ .name | indent 2 }}: |-
{{ .content | indent 4 }}
{{ end -}}
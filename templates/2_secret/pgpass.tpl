{{ if eq .cluster.values.security.password.method "operator" }}
apiVersion: v1
kind: Secret
metadata:
  name:  {{ $app_name }}-{{ .cluster.values.security.password.secret_name }}
  labels:
    app.kubernetes.io/component: {{ .cluster.values.security.password.secret_name }}
{{- template "global_labels" . }}
type: Opaque
data:
{{ range .cluster.generated_passwords -}}
{{ .secret_key | indent 2 }}: |-
{{ .secret_value | indent 4 }}
{{ end -}}

{{- end -}}
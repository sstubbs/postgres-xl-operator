apiVersion: v1
kind: Service
metadata:
  name: {{ $app_name }}-svc
  labels:
    app.kubernetes.io/component: external
{{- template "global_labels" . }}
spec:
  type: {{ .cluster.values.service.service_type }}
  ports:
  - port: {{ .cluster.values.config.postgres_port }}
    protocol: TCP
    targetPort: {{ .cluster.values.config.postgres_port }}
    name: pg
  selector:
    app.kubernetes.io/component: crd
{{ range .cluster.selector_labels -}}
{{ .name | indent 4 }}: {{ .content }}
{{ end }}
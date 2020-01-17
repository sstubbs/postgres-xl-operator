apiVersion: v1
kind: Service
metadata:
  name: {{ $app_name }}-svc-gtm
  labels:
    app.kubernetes.io/component: svc-gtm
{{- template "global_labels" . }}
spec:
  clusterIP: None
  ports:
  - port: {{ .cluster.values.config.managers_port }}
    protocol: TCP
    targetPort: {{ .cluster.values.config.managers_port }}
    name: gtm
  selector:
    app: {{ $app_name }}
    type: gtm
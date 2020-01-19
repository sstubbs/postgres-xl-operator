{{- $component := "pxy" -}}

apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: {{ $app_name }}-{{ $component }}
  labels:
    app.kubernetes.io/component: {{ $component }}
{{- template "global_labels" . }}
spec:
  serviceName: {{ $app_name }}-svc-{{ $component }}
  replicas: {{ .cluster.values.proxies.count }}
  podManagementPolicy: Parallel
  selector:
    matchLabels:
      app.kubernetes.io/component: {{ $component }}
{{ range .cluster.selector_labels -}}
{{ .name | indent 6 }}: {{ .content }}
{{ end }}
{{- if .cluster.values.proxies.inject_sts_yaml }}
{{ .cluster.values.proxies.inject_sts_yaml | indent 2 }}
{{- end }}
  template:
    metadata:
      labels:
        app.kubernetes.io/component: {{ $component }}
{{ range .cluster.selector_labels -}}
{{ .name | indent 8 }}: {{ .content }}
{{ end }}
    spec:
{{- if .cluster.values.proxies.inject_spec_yaml }}
{{ .cluster.values.proxies.inject_spec_yaml | indent 6 }}
{{- end }}
      containers:
      - name: {{ $component }}
        image: {{ .cluster.values.image.name }}:{{ .cluster.values.image.version }}
        command:
          - bash
          - /scripts/proxy_entrypoint
        env:
          - name: POD_NAME
            valueFrom:
              fieldRef:
                fieldPath: metadata.name
          - name: POD_IP
            valueFrom:
              fieldRef:
                fieldPath: status.podIP
          - name: NODE_TYPE
            value: proxy
        envFrom:
        - configMapRef:
            name: {{ $app_name }}-envs
        ports:
          - containerPort: {{ .cluster.values.config.managers_port }}
            name: {{ $component }}
        readinessProbe:
          exec:
            command:
            - /scripts/probe_readiness_proxy
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
{{- if or .cluster.values.proxies.resources.requests.memory .cluster.values.proxies.resources.requests.cpu }}
          requests:
{{- end }}
{{- if .cluster.values.proxies.resources.requests.memory }}
            memory: {{ .cluster.values.proxies.resources.requests.memory }}
{{- end }}
{{- if .cluster.values.proxies.resources.requests.cpu }}
            cpu: {{ .cluster.values.proxies.resources.requests.cpu }}
{{- end }}
{{- if or .cluster.values.proxies.resources.limits.memory .cluster.values.proxies.resources.limits.cpu }}
          limits:
{{- end }}
{{- if .cluster.values.proxies.resources.limits.memory }}
            memory: {{ .cluster.values.proxies.resources.limits.memory }}
{{- end }}
{{- if .cluster.values.proxies.resources.limits.cpu }}
            cpu: {{ .cluster.values.proxies.resources.limits.cpu }}
{{- end }}
        volumeMounts:
          - name: {{ $app_name }}-scripts
            mountPath: /scripts
          - name: {{ $app_name }}-cfg
            mountPath: /config
          - name: datastore
            mountPath: {{ .cluster.values.homedir }}/storage
{{- if .cluster.values.proxies.volume_mounts }}
{{ .cluster.values.proxies.volume_mounts | indent 10 }}
{{- end }}
{{- if .cluster.values.proxies.inject_main_container_yaml }}
{{ .cluster.values.proxies.inject_main_container_yaml | indent 8 }}
{{- end }}
{{- if .cluster.values.proxies.add_containers }}
{{ .cluster.values.proxies.add_containers | indent 6 }}
{{- end }}
      volumes:
        - name: datastore
          emptyDir: {}
        - name: {{ $app_name }}-scripts
          configMap:
            name: {{ $app_name }}-scripts
            defaultMode: 511
        - name: {{ $app_name }}-cfg
          configMap:
            name: {{ $app_name }}-cfg
            defaultMode: 511
{{- if .cluster.values.proxies.volumes }}
{{ .cluster.values.proxies.volumes | indent 8 }}
{{- end }}
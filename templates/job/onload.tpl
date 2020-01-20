{{- $component := "onload" -}}

# Main condition.
{{ if .cluster.values.on_load.enabled }}

apiVersion: batch/v1
kind: Job
metadata:
  name: {{ $app_name }}-{{ $component }}
  labels:
    app.kubernetes.io/component: {{ $component }}
{{- template "global_labels" . }}
spec:
  backoffLimit: {{ .cluster.values.on_load.back_off_limit }}
  template:
    metadata:
      labels:
        app.kubernetes.io/component: {{ $component }}
{{ range .cluster.selector_labels -}}
{{ .name | indent 8 }}: {{ .content }}
{{ end }}
{{- if .cluster.values.on_load.inject_job_yaml }}
{{ .cluster.values.on_load.inject_job_yaml | indent 2 }}
{{- end }}
    spec:
      restartPolicy: OnFailure
{{- if .cluster.values.on_load.inject_spec_yaml }}
{{ .cluster.values.on_load.inject_spec_yaml | indent 6 }}
{{- end }}
      containers:
      - name: {{ $component }}
        image: {{ .cluster.values.image.name }}:{{ .cluster.values.image.version }}
        command:
          - bash
          - /scripts/job_on_load
        envFrom:
        - configMapRef:
            name: {{ $app_name }}-envs
        resources:
{{- if or .cluster.values.on_load.resources.requests.memory .cluster.values.on_load.resources.requests.cpu }}
          requests:
{{- end }}
{{- if .cluster.values.on_load.resources.requests.memory }}
            memory: {{ .cluster.values.on_load.resources.requests.memory }}
{{- end }}
{{- if .cluster.values.on_load.resources.requests.cpu }}
            cpu: {{ .cluster.values.on_load.resources.requests.cpu }}
{{- end }}
{{- if or .cluster.values.on_load.resources.limits.memory .cluster.values.on_load.resources.limits.cpu }}
          limits:
{{- end }}
{{- if .cluster.values.on_load.resources.limits.memory }}
            memory: {{ .cluster.values.on_load.resources.limits.memory }}
{{- end }}
{{- if .cluster.values.on_load.resources.limits.cpu }}
            cpu: {{ .cluster.values.on_load.resources.limits.cpu }}
{{- end }}
        env:
          - name: POD_NAME
            valueFrom:
              fieldRef:
                fieldPath: metadata.name
          - name: POD_IP
            valueFrom:
              fieldRef:
                fieldPath: status.podIP
          - name: PGHOST
            value: "{{ $app_name }}-svc"
{{- template "global_secret" . }}
        volumeMounts:
          - name: {{ $app_name }}-{{ $component }}
            mountPath: /load_scripts
          - name: {{ $app_name }}-scripts
            mountPath: /scripts
{{- if .cluster.values.on_load.volume_mounts }}
{{ .cluster.values.on_load.volume_mounts | indent 10 }}
{{- end }}
{{- if .cluster.values.on_load.inject_main_container_yaml }}
{{ .cluster.values.on_load.inject_main_container_yaml | indent 8 }}
{{- end }}
{{- if .cluster.values.on_load.add_containers }}
{{ .cluster.values.on_load.add_containers | indent 6 }}
{{- end }}
      volumes:
        - name: {{ $app_name }}-scripts
          configMap:
            name: {{ $app_name }}-scripts
            defaultMode: 511
        - name: {{ $app_name }}-{{ $component }}
          configMap:
            name: {{ $app_name }}-{{ $component }}
            defaultMode: 511
{{- if .cluster.values.on_load.volumes }}
{{ .cluster.values.on_load.volumes | indent 8 }}
{{- end }}

# End of main condition.
{{- end }}
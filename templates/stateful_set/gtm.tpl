apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: {{ $app_name }}-gtm
  labels:
    app.kubernetes.io/component: gtm
{{- template "global_labels" . }}
spec:
  serviceName: {{ $app_name }}-svc-gm
  replicas: 1
  podManagementPolicy: Parallel
  volumeClaimTemplates:
{{- if .cluster.values.gtm.pvc.resources.requests.storage }}
    - metadata:
        name: datastore
      spec:
        pvc:
          resources:
            requests:
              storage: {{ .cluster.values.gtm.pvc.resources.requests.storage }}
{{- end }}
{{- if .cluster.values.gtm.add_volume_claims }}
{{ .cluster.values.gtm.add_volume_claims | indent 4 }}
{{- end }}
  selector:
    matchLabels:
      app: {{ $app_name }}
      type: gtm
{{- if .cluster.values.gtm.inject_sts_yaml }}
{{ .cluster.values.gtm.inject_sts_yaml | indent 2 }}
{{- end }}
  template:
    metadata:
      labels:
        app: {{ $app_name }}
        type: gtm
    spec:
      securityContext:
        fsGroup: 3000
{{- if .cluster.values.gtm.inject_spec_yaml }}
{{ .cluster.values.gtm.inject_spec_yaml | indent 6 }}
{{- end }}
      containers:
      - name: gtm
        image: {{ .cluster.values.image.name }}:{{ .cluster.values.image.version }}
        command:
          - bash
          - /scripts/gtm_entrypoint
        envFrom:
        - configMapRef:
            name: {{$app_name}}-envs
        ports:
          - containerPort: {{ .cluster.values.config.managers_port }}
            name: gtm
        resources:
{{- if .cluster.values.gtm.resources.requests.memory }}
{{- if .cluster.values.gtm.resources.requests.cpu }}
          requests:
{{- if .cluster.values.gtm.resources.requests.memory }}
            memory: {{ .cluster.values.gtm.resources.requests.memory }}
{{- end }}
{{- if .cluster.values.gtm.resources.requests.cpu }}
            cpu: {{ .cluster.values.gtm.resources.requests.cpu }}
{{- end }}
{{- end }}
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
          - name: NODE_TYPE
            value: gtm
        volumeMounts:
          - name: {{ $app_name }}-scripts
            mountPath: /scripts
          - name: {{ $app_name }}-cfg
            mountPath: /config
          - name: datastore
            mountPath: {{ .cluster.values.homedir }}/storage
{{- if .cluster.values.gtm.volume_mounts }}
{{ .cluster.values.gtm.volume_mounts | indent 10 }}
{{- end }}
{{- if .cluster.values.gtm.inject_main_container_yaml }}
{{ .cluster.values.gtm.inject_main_container_yaml | indent 8 }}
{{- end }}
{{- if .cluster.values.gtm.add_containers }}
{{ .cluster.values.gtm.add_containers | indent 6 }}
{{- end }}
      volumes:
{{- if .cluster.values.gtm.pvc.resources.requests.storage }}{{- else }}
        - name: datastore
          emptyDir: {}
{{- end }}
        - name: {{ $app_name }}-scripts
          configMap:
            name: {{ $app_name }}-scripts
            defaultMode: 0777
        - name: {{ $app_name }}-cfg
          configMap:
            name: {{ $app_name }}-cfg
            defaultMode: 0777
{{- if .cluster.values.gtm.volumes }}
{{ .cluster.values.gtm.add_containers | indent 8 }}
{{- end }}
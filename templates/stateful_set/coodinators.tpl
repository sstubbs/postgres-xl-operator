{{- $component := "crd" -}}

apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: {{ $app_name }}-{{ $component }}
  labels:
    app.kubernetes.io/component: {{ $component }}
{{- template "global_labels" . }}
spec:
  serviceName: {{ $app_name }}-svc-{{ $component }}
  replicas: {{ .cluster.values.coordinators.count }}
  podManagementPolicy: Parallel
  volumeClaimTemplates:
{{- if .cluster.values.coordinators.pvc.resources.requests.storage }}
    - metadata:
        name: datastore
      spec:
        accessModes: [ "ReadWriteOnce" ]
        pvc:
          resources:
            requests:
              storage: {{ .cluster.values.coordinators.pvc.resources.requests.storage }}
{{- end }}
{{- if .cluster.values.coordinators.add_volume_claims }}
{{ .cluster.values.coordinators.add_volume_claims | indent 4 }}
{{- end }}
  selector:
    matchLabels:
      app.kubernetes.io/component: {{ $component }}
{{ range .cluster.selector_labels -}}
{{ .name | indent 6 }}: {{ .content }}
{{ end }}
{{- if .cluster.values.coordinators.inject_sts_yaml }}
{{ .cluster.values.coordinators.inject_sts_yaml | indent 2 }}
{{- end }}
  template:
    metadata:
      labels:
        app.kubernetes.io/component: {{ $component }}
{{ range .cluster.selector_labels -}}
{{ .name | indent 8 }}: {{ .content }}
{{ end }}
    spec:
      securityContext:
        fsGroup: 3000
{{- if .cluster.values.coordinators.inject_spec_yaml }}
{{ .cluster.values.coordinators.inject_spec_yaml | indent 6 }}
{{- end }}
      containers:
      - name: {{ $component }}
        image: {{ .cluster.values.image.name }}:{{ .cluster.values.image.version }}
        command:
          - bash
          - /scripts/data_node_entrypoint
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
            value: coordinator
        envFrom:
        - configMapRef:
            name: {{ $app_name }}-envs
        ports:
          - containerPort: {{ .cluster.values.config.postgres_port }}
            name: {{ $component }}
        readinessProbe:
          exec:
            command:
            - /scripts/probe_readiness_coordinator
          initialDelaySeconds: 5
          periodSeconds: 5
        livenessProbe:
          exec:
            command:
            - /scripts/probe_liveness_postgres
          initialDelaySeconds: 30
          periodSeconds: 5
        resources:
{{- if or .cluster.values.coordinators.resources.requests.memory .cluster.values.coordinators.resources.requests.cpu }}
          requests:
{{- end }}
{{- if .cluster.values.coordinators.resources.requests.memory }}
            memory: {{ .cluster.values.coordinators.resources.requests.memory }}
{{- end }}
{{- if .cluster.values.coordinators.resources.requests.cpu }}
            cpu: {{ .cluster.values.coordinators.resources.requests.cpu }}
{{- end }}
{{- if or .cluster.values.coordinators.resources.limits.memory .cluster.values.coordinators.resources.limits.cpu }}
          limits:
{{- end }}
{{- if .cluster.values.coordinators.resources.limits.memory }}
            memory: {{ .cluster.values.coordinators.resources.limits.memory }}
{{- end }}
{{- if .cluster.values.coordinators.resources.limits.cpu }}
            cpu: {{ .cluster.values.coordinators.resources.limits.cpu }}
{{- end }}
        volumeMounts:
          - name: {{ $app_name }}-scripts
            mountPath: /scripts
          - name: {{ $app_name }}-cfg
            mountPath: /config
          - name: datastore
            mountPath: {{ .cluster.values.homedir }}/storage
{{- if .cluster.values.coordinators.volume_mounts }}
{{ .cluster.values.coordinators.volume_mounts | indent 10 }}
{{- end }}
{{- if .cluster.values.coordinators.inject_main_container_yaml }}
{{ .cluster.values.coordinators.inject_main_container_yaml | indent 8 }}
{{- end }}
{{- if .cluster.values.coordinators.add_containers }}
{{ .cluster.values.coordinators.add_containers | indent 6 }}
{{- end }}
      volumes:
{{- if .cluster.values.coordinators.pvc.resources.requests.storage }}{{- else }}
        - name: datastore
          emptyDir: {}
{{- end }}
        - name: {{ $app_name }}-scripts
          configMap:
            name: {{ $app_name }}-scripts
            defaultMode: 511
        - name: {{ $app_name }}-cfg
          configMap:
            name: {{ $app_name }}-cfg
            defaultMode: 511
{{- if .cluster.values.coordinators.volumes }}
{{ .cluster.values.coordinators.volumes | indent 8 }}
{{- end }}
{{ range .cluster.scripts -}}
{{ indent 4 .name }}: |
{{ indent 8 .script }}
{{ end }}
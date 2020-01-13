{{- define "tmpl1"}} some {{ end -}}
{{- define "tmpl2"}} some other {{ end -}}
there is {{template ("tmpl1")}} template
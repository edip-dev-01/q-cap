package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"path/filepath"
)

type Capsule struct {
	Name        string `json:"name"`
	Path        string `json:"path"`
	Size        int64  `json:"size"`
	ContentType string `json:"content_type"`
}

type Registry struct {
	SeedDir string
	Index   []Capsule
}

func (r *Registry) loadIndex() error {
	var idx []Capsule
	err := filepath.Walk(r.SeedDir, func(p string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			return nil
		}
		if filepath.Ext(p) == ".qcap" {
			name := filepath.Base(p)
			idx = append(idx, Capsule{
				Name:        name,
				Path:        "/artifacts/" + name,
				Size:        info.Size(),
				ContentType: "application/qcap+zip",
			})
		}
		return nil
	})
	if err != nil {
		return err
	}
	r.Index = idx
	return nil
}

func main() {
	seedDir := os.Getenv("QCAP_REGISTRY_SEED")
	if seedDir == "" {
		seedDir = "services/qcap-registry/seed"
	}
	reg := &Registry{SeedDir: seedDir}
	if err := reg.loadIndex(); err != nil {
		log.Printf("warn: could not load seed: %v", err)
	}

	mux := http.NewServeMux()

	// Root landing page
	mux.HandleFunc("/", func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`<!doctype html><html><head><meta charset="utf-8"><title>QCAP Registry</title></head><body>
							<h1>QCAP Registry (demo)</h1>
							<p>This is a minimal, read-only registry for demo purposes. It serves a seeded list of <code>.qcap</code> capsules from a local directory. Use the links below to explore.</p>
				<ul>
								<li><a href="/health">/health</a> (JSON status)</li>
								<li><a href="/health.html">/health.html</a> (HTML status)</li>
								<li><a href="/index.json">/index.json</a> (JSON index)</li>
								<li><a href="/index">/index</a> (HTML index)</li>
								<li><a href="/artifacts/">/artifacts/</a> (static downloads)</li>
				</ul>
			</body></html>`))
	})

	// JSON health endpoint
	mux.HandleFunc("/health", func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"status":"ok"}`))
	})

	// HTML health page with description
	mux.HandleFunc("/health.html", func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`<!doctype html><html><head><meta charset="utf-8"><title>Health</title></head><body>
				<h1>Health</h1>
				<p>This endpoint reports the registry status. If you see "ok", the server is running.</p>
				<pre>{"status":"ok"}</pre>
				<p>For machine-readable output, use <a href="/health">/health</a>.</p>
			</body></html>`))
	})

	// JSON index endpoint
	mux.HandleFunc("/index.json", func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(reg.Index)
	})

	// HTML index page with description and simple listing
	mux.HandleFunc("/index", func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("<!doctype html><html><head><meta charset=\"utf-8\"><title>Index</title></head><body>"))
		w.Write([]byte("<h1>Artifact Index</h1>\n"))
		w.Write([]byte("<p>This page lists the seeded .qcap capsules available for download. Use the links to retrieve artifacts.</p>"))
		if len(reg.Index) == 0 {
			w.Write([]byte("<p><em>No artifacts found in seed directory.</em></p>"))
		} else {
			w.Write([]byte("<ul>"))
			for _, c := range reg.Index {
				w.Write([]byte("<li>"))
				w.Write([]byte(c.Name))
				w.Write([]byte(" — <a href=\"" + c.Path + "\">download</a> (" + c.ContentType + ", " + formatSize(c.Size) + ")"))
				w.Write([]byte("</li>"))
			}
			w.Write([]byte("</ul>"))
		}
		w.Write([]byte("<p>Machine-readable index: <a href=\"/index.json\">/index.json</a></p>"))
		w.Write([]byte("</body></html>"))
	})

	mux.Handle("/artifacts/", http.StripPrefix("/artifacts/", http.FileServer(http.Dir(seedDir))))

	addr := ":8080"
	log.Printf("registry listening on %s; seed dir: %s", addr, seedDir)
	log.Fatal(http.ListenAndServe(addr, mux))
}

// formatSize returns a human-readable byte size (e.g., 1.2 MB)
func formatSize(n int64) string {
	const (
		KB = 1024
		MB = KB * 1024
		GB = MB * 1024
	)
	switch {
	case n >= GB:
		return fmt.Sprintf("%.2f GB", float64(n)/float64(GB))
	case n >= MB:
		return fmt.Sprintf("%.2f MB", float64(n)/float64(MB))
	case n >= KB:
		return fmt.Sprintf("%.2f KB", float64(n)/float64(KB))
	default:
		return fmt.Sprintf("%d B", n)
	}
}

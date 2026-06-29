package main

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"
)

func TestPublishRequiresBearerTokenWhenConfigured(t *testing.T) {
	dir := t.TempDir()
	reg := &Registry{
		StoreDir:  dir,
		IndexPath: filepath.Join(dir, "index.json"),
		Token:     "secret",
	}

	req := httptest.NewRequest(http.MethodPost, "/artifacts", bytes.NewReader([]byte("qcap")))
	req.Header.Set("X-Qcap-Name", "demo.qcap")
	rec := httptest.NewRecorder()
	reg.publish(rec, req)

	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected unauthorized, got %d", rec.Code)
	}
	if _, err := os.Stat(filepath.Join(dir, "demo.qcap")); !os.IsNotExist(err) {
		t.Fatalf("artifact should not be written without auth")
	}
}

func TestPublishPersistsIndex(t *testing.T) {
	dir := t.TempDir()
	reg := &Registry{
		StoreDir:  dir,
		IndexPath: filepath.Join(dir, "index.json"),
		Token:     "secret",
	}

	req := httptest.NewRequest(http.MethodPost, "/artifacts", bytes.NewReader([]byte("qcap")))
	req.Header.Set("Authorization", "Bearer secret")
	req.Header.Set("X-Qcap-Name", "demo.qcap")
	rec := httptest.NewRecorder()
	reg.publish(rec, req)

	if rec.Code != http.StatusCreated {
		t.Fatalf("expected created, got %d: %s", rec.Code, rec.Body.String())
	}
	if _, err := os.Stat(filepath.Join(dir, "demo.qcap")); err != nil {
		t.Fatalf("artifact not written: %v", err)
	}

	var persisted []Capsule
	bytes, err := os.ReadFile(filepath.Join(dir, "index.json"))
	if err != nil {
		t.Fatalf("index not written: %v", err)
	}
	if err := json.Unmarshal(bytes, &persisted); err != nil {
		t.Fatalf("index invalid JSON: %v", err)
	}
	if len(persisted) != 1 {
		t.Fatalf("expected one capsule, got %d", len(persisted))
	}
	if persisted[0].Name != "demo.qcap" {
		t.Fatalf("unexpected capsule name: %s", persisted[0].Name)
	}
	if persisted[0].Digest == "" || persisted[0].CreatedAt == "" {
		t.Fatalf("expected digest and created_at in persisted index: %+v", persisted[0])
	}
}

package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"
)

type Task struct {
	ID          int    `json:"id"`
	Description string `json:"description"`
	Status      string `json:"status"`
	CreatedAt   string `json:"createdAt"`
	UpdatedAt   string `json:"updatedAt"`
}

var validStatuses = map[string]bool{
	"todo":        true,
	"in-progress": true,
	"done":        true,
}

func dataFilePath() string {
	cwd, err := os.Getwd()
	if err != nil {
		exitErr("Failed to determine current directory.")
	}
	return filepath.Join(cwd, "tasks.json")
}

func exitErr(msg string) {
	fmt.Fprintln(os.Stderr, msg)
	os.Exit(1)
}

func readTasks() []Task {
	file := dataFilePath()
	if _, err := os.Stat(file); os.IsNotExist(err) {
		if err := os.WriteFile(file, []byte("[]"), 0644); err != nil {
			exitErr("Failed to create tasks.json.")
		}
		return []Task{}
	}

	raw, err := os.ReadFile(file)
	if err != nil {
		exitErr("Failed to read tasks.json.")
	}
	if strings.TrimSpace(string(raw)) == "" {
		return []Task{}
	}

	var tasks []Task
	if err := json.Unmarshal(raw, &tasks); err != nil {
		exitErr("Invalid JSON format in tasks.json.")
	}
	return tasks
}

func writeTasks(tasks []Task) {
	file := dataFilePath()
	raw, err := json.MarshalIndent(tasks, "", "  ")
	if err != nil {
		exitErr("Failed to serialize tasks.")
	}
	if err := os.WriteFile(file, raw, 0644); err != nil {
		exitErr("Failed to write tasks.json.")
	}
}

func parseID(raw string) int {
	id, err := strconv.Atoi(raw)
	if err != nil || id <= 0 {
		exitErr("Invalid task ID. It must be a positive integer.")
	}
	return id
}

func nextID(tasks []Task) int {
	max := 0
	for _, t := range tasks {
		if t.ID > max {
			max = t.ID
		}
	}
	return max + 1
}

func usage() {
	fmt.Println(`Task Tracker CLI (Go)

Usage:
  go run task-cli.go add "Task description"
  go run task-cli.go update <id> "New description"
  go run task-cli.go delete <id>
  go run task-cli.go mark-in-progress <id>
  go run task-cli.go mark-done <id>
  go run task-cli.go list
  go run task-cli.go list <todo|in-progress|done|not-done>`)
}

func printTasks(tasks []Task) {
	if len(tasks) == 0 {
		fmt.Println("No tasks found.")
		return
	}
	for _, t := range tasks {
		fmt.Printf("[%d] %s | status: %s | createdAt: %s | updatedAt: %s\n", t.ID, t.Description, t.Status, t.CreatedAt, t.UpdatedAt)
	}
}

func addTask(description string) {
	desc := strings.TrimSpace(description)
	if desc == "" {
		exitErr("Task description cannot be empty.")
	}
	tasks := readTasks()
	now := time.Now().UTC().Format(time.RFC3339Nano)
	task := Task{
		ID:          nextID(tasks),
		Description: desc,
		Status:      "todo",
		CreatedAt:   now,
		UpdatedAt:   now,
	}
	tasks = append(tasks, task)
	writeTasks(tasks)
	fmt.Printf("Task added successfully (ID: %d)\n", task.ID)
}

func updateTask(id int, description string) {
	desc := strings.TrimSpace(description)
	if desc == "" {
		exitErr("Updated description cannot be empty.")
	}
	tasks := readTasks()
	for i := range tasks {
		if tasks[i].ID == id {
			tasks[i].Description = desc
			tasks[i].UpdatedAt = time.Now().UTC().Format(time.RFC3339Nano)
			writeTasks(tasks)
			fmt.Printf("Task %d updated successfully.\n", id)
			return
		}
	}
	exitErr(fmt.Sprintf("Task with ID %d was not found.", id))
}

func deleteTask(id int) {
	tasks := readTasks()
	filtered := make([]Task, 0, len(tasks))
	found := false
	for _, t := range tasks {
		if t.ID == id {
			found = true
			continue
		}
		filtered = append(filtered, t)
	}
	if !found {
		exitErr(fmt.Sprintf("Task with ID %d was not found.", id))
	}
	writeTasks(filtered)
	fmt.Printf("Task %d deleted successfully.\n", id)
}

func setStatus(id int, status string) {
	tasks := readTasks()
	for i := range tasks {
		if tasks[i].ID == id {
			tasks[i].Status = status
			tasks[i].UpdatedAt = time.Now().UTC().Format(time.RFC3339Nano)
			writeTasks(tasks)
			fmt.Printf("Task %d marked as %s.\n", id, status)
			return
		}
	}
	exitErr(fmt.Sprintf("Task with ID %d was not found.", id))
}

func listTasks(filter string) {
	tasks := readTasks()
	if filter == "" {
		printTasks(tasks)
		return
	}
	if filter == "not-done" {
		filtered := make([]Task, 0)
		for _, t := range tasks {
			if t.Status != "done" {
				filtered = append(filtered, t)
			}
		}
		printTasks(filtered)
		return
	}
	if !validStatuses[filter] {
		exitErr("Invalid status filter. Use: todo, in-progress, done, not-done")
	}
	filtered := make([]Task, 0)
	for _, t := range tasks {
		if t.Status == filter {
			filtered = append(filtered, t)
		}
	}
	printTasks(filtered)
}

func main() {
	args := os.Args[1:]
	if len(args) == 0 || args[0] == "help" || args[0] == "--help" || args[0] == "-h" {
		usage()
		return
	}

	switch args[0] {
	case "add":
		if len(args) < 2 {
			exitErr(`Usage: go run task-cli.go add "Task description"`)
		}
		addTask(args[1])
	case "update":
		if len(args) < 3 {
			exitErr(`Usage: go run task-cli.go update <id> "New description"`)
		}
		updateTask(parseID(args[1]), args[2])
	case "delete":
		if len(args) < 2 {
			exitErr("Usage: go run task-cli.go delete <id>")
		}
		deleteTask(parseID(args[1]))
	case "mark-in-progress":
		if len(args) < 2 {
			exitErr("Usage: go run task-cli.go mark-in-progress <id>")
		}
		setStatus(parseID(args[1]), "in-progress")
	case "mark-done":
		if len(args) < 2 {
			exitErr("Usage: go run task-cli.go mark-done <id>")
		}
		setStatus(parseID(args[1]), "done")
	case "list":
		if len(args) > 2 {
			exitErr("Usage: go run task-cli.go list [todo|in-progress|done|not-done]")
		}
		filter := ""
		if len(args) == 2 {
			filter = args[1]
		}
		listTasks(filter)
	default:
		exitErr("Unknown command. Run with --help for usage.")
	}
}

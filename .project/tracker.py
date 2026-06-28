#!/usr/bin/env python3
# .project/tracker.py — Project session tracker CLI
# All markdown files are written by this tool, never hand-edited by AI.
# Usage: python .project/tracker.py <command> [args]
#
# Commands:
#   status              Show current state
#   start <task>        Begin a task (TODO -> DOING)
#   finish [summary]    End current session
#   done <task>         Move task DOING -> DONE
#   decide "<title>" "<reason>" "<impact>"
#                       Add an architecture decision
#   state <key> <value> Update CURRENT_STATE field
#   todo <task>         Add a task to TODO

import json, os, sys, datetime, subprocess
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
PROJECT_DIR = PROJECT_ROOT / '.project'

STATE_FILE = PROJECT_DIR / 'state.json'
TASKS_FILE = PROJECT_DIR / 'tasks.json'
DECISIONS_FILE = PROJECT_DIR / 'decisions.json'
SESSION_LOG = PROJECT_DIR / 'SESSION_LOG.md'
CURRENT_STATE = PROJECT_DIR / 'CURRENT_STATE.md'
TASK_BOARD = PROJECT_DIR / 'TASK_BOARD.md'
DECISIONS_MD = PROJECT_DIR / 'DECISIONS.md'

def load_json(path):
    if path.exists():
        return json.loads(path.read_text(encoding='utf-8'))
    return {}

def save_json(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding='utf-8')

def now():
    return datetime.datetime.now().strftime('%Y-%m-%d %H:%M')

def current_branch():
    try:
        r = subprocess.run(['git','branch','--show-current'], capture_output=True, text=True, cwd=PROJECT_ROOT)
        return r.stdout.strip() or 'unknown'
    except:
        return 'unknown'

def render_current_state():
    s = load_json(STATE_FILE)
    lines = [
        '# Current State',
        '',
        f'**Branch**: {s.get("branch",current_branch())}',
        '',
        '## Active Work',
    ]
    for task in s.get('active_work', []):
        status = task.get('status','?')
        name = task.get('name','?')
        emoji = {'done':'[x]','in_progress':'[~]','todo':'[ ]'}.get(status, '[ ]')
        lines.append(f'- {emoji} {name}')
    lines.append('')
    lines.append('## Recent Changes')
    for f in s.get('recent_files', []):
        lines.append(f'- {f}')
    if not s.get('recent_files'):
        lines.append('- (none)')
    lines.append('')
    lines.append('## Known Issues')
    for i in s.get('known_issues', []):
        lines.append(f'- {i}')
    if not s.get('known_issues'):
        lines.append('- (none)')
    lines.append('')
    lines.append('## Next Step')
    nx = s.get('next_step','')
    lines.append(f'- {nx}' if nx else '- (not set)')
    CURRENT_STATE.write_text('\n'.join(lines)+'\n', encoding='utf-8')

def render_task_board():
    tasks = load_json(TASKS_FILE)
    columns = {'todo': [], 'doing': [], 'done': []}
    for t in tasks.get('tasks', []):
        col = t.get('status','todo')
        columns.setdefault(col, []).append(t['name'])
    lines = ['# Task Board', '']
    for col, emoji in [('todo','TODO'),('doing','DOING'),('done','DONE')]:
        lines.append(f'## {emoji}')
        for name in columns.get(col,[]):
            lines.append(f'- {name}')
        if not columns.get(col,[]):
            lines.append('- (empty)')
        lines.append('')
    TASK_BOARD.write_text('\n'.join(lines)+'\n', encoding='utf-8')

def render_decisions():
    decs = load_json(DECISIONS_FILE)
    lines = ['# Architecture Decisions', '']
    for i, d in enumerate(decs.get('decisions',[]), 1):
        lines.append(f'## ADR-{i:03d}: {d["title"]}')
        lines.append('')
        lines.append(f'**Reason**: {d["reason"]}')
        lines.append('')
        lines.append(f'**Impact**: {d["impact"]}')
        lines.append('')
        lines.append(f'*{d.get("date","")}*')
        lines.append('')
    if not decs.get('decisions'):
        lines.append('- (no decisions recorded)')
    DECISIONS_MD.write_text('\n'.join(lines)+'\n', encoding='utf-8')

def render_all():
    render_current_state()
    render_task_board()
    render_decisions()

def cmd_status():
    print('=== Branch:', current_branch())
    s = load_json(STATE_FILE)
    print('Active work:', s.get('active_work',[]))
    print('Next step:', s.get('next_step','(not set)'))
    tasks = load_json(TASKS_FILE)
    doing = [t['name'] for t in tasks.get('tasks',[]) if t.get('status')=='doing']
    print('Currently DOING:', doing or '(none)')
    print('Known issues:', s.get('known_issues',[]) or '(none)')

def cmd_start(task_name):
    branch = current_branch()
    tasks = load_json(TASKS_FILE)
    task_list = tasks.get('tasks', [])

    # Check if another session has this task
    for t in task_list:
        if t['name'] == task_name and t['status'] == 'doing':
            print(f'WARNING: Task "{task_name}" is already DOING in another session!')
            return

    # Move from TODO to DOING
    for t in task_list:
        if t['name'] == task_name and t['status'] == 'todo':
            t['status'] = 'doing'
            t['started'] = now()
            break
    else:
        task_list.append({'name': task_name, 'status': 'doing', 'started': now()})

    tasks['tasks'] = task_list
    save_json(TASKS_FILE, tasks)

    # Update state
    s = load_json(STATE_FILE)
    s['branch'] = branch
    s.setdefault('active_work', [])
    found = False
    for w in s['active_work']:
        if w['name'] == task_name:
            w['status'] = 'in_progress'
            found = True
    if not found:
        s['active_work'].append({'name': task_name, 'status': 'in_progress'})
    s['next_step'] = f'Complete: {task_name}'
    save_json(STATE_FILE, s)
    render_all()
    print(f'Started: {task_name} on {branch} at {now()}')

def cmd_finish(summary=''):
    branch = current_branch()
    s = load_json(STATE_FILE)
    tasks = load_json(TASKS_FILE)

    # Get current active task
    doing = [t for t in tasks.get('tasks',[]) if t.get('status')=='doing']
    task_name = doing[0]['name'] if doing else 'unnamed task'

    # Append session log
    entry = f"""### [{now()}] {branch} — {task_name}

- **Task**: {task_name}
- **Branch**: {branch}
- **Summary**: {summary or '(no summary)'}
- **Next**: {s.get('next_step','(not set)')}

"""
    with open(SESSION_LOG, 'a', encoding='utf-8') as f:
        f.write(entry)

    # Move task to done
    for t in tasks.get('tasks',[]):
        if t['status'] == 'doing':
            t['status'] = 'done'
            t['finished'] = now()

    save_json(TASKS_FILE, tasks)

    # Update active work
    for w in s.get('active_work',[]):
        if w['status'] == 'in_progress':
            w['status'] = 'done'
    save_json(STATE_FILE, s)
    render_all()
    print(f'Finished: {task_name} at {now()}')
    print(f'Session log appended: {SESSION_LOG}')

def cmd_done(task_name):
    tasks = load_json(TASKS_FILE)
    for t in tasks.get('tasks',[]):
        if t['name'] == task_name:
            t['status'] = 'done'
            t['finished'] = now()
    save_json(TASKS_FILE, tasks)
    render_task_board()
    print(f'Done: {task_name}')

def cmd_decide(title, reason, impact):
    decs = load_json(DECISIONS_FILE)
    decs.setdefault('decisions', []).append({
        'title': title,
        'reason': reason,
        'impact': impact,
        'date': now()
    })
    save_json(DECISIONS_FILE, decs)
    render_decisions()
    n = len(decs['decisions'])
    print(f'ADR-{n:03d} added: {title}')

def cmd_state(key, value):
    s = load_json(STATE_FILE)
    if key == 'branch':
        s['branch'] = value
    elif key == 'next_step':
        s['next_step'] = value
    elif key == 'issue':
        issues = s.setdefault('known_issues', [])
        issues.append(value)
    elif key == 'file':
        files = s.setdefault('recent_files', [])
        if value not in files:
            files.insert(0, value)
            s['recent_files'] = files[:10]
    else:
        print(f'Unknown key: {key}')
        return
    save_json(STATE_FILE, s)
    render_current_state()
    print(f'State updated: {key} = {value}')

def cmd_todo(task_name):
    tasks = load_json(TASKS_FILE)
    tasks.setdefault('tasks', []).append({'name': task_name, 'status': 'todo'})
    save_json(TASKS_FILE, tasks)
    render_task_board()
    print(f'TODO added: {task_name}')

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print('Usage: tracker.py <status|start|finish|done|decide|state|todo> [args]')
        sys.exit(1)
    cmd = sys.argv[1]
    if cmd == 'init':
        PROJECT_DIR.mkdir(parents=True, exist_ok=True)
        for f in [STATE_FILE, TASKS_FILE, DECISIONS_FILE]:
            if not f.exists():
                save_json(f, {})
        render_all()
        if not SESSION_LOG.exists():
            SESSION_LOG.write_text('# Session Log\n\n> Append-only timeline. Never edit past entries.\n\n', encoding='utf-8')
        print('Tracker initialized.')
    elif cmd == 'status':
        cmd_status()
    elif cmd == 'start' and len(sys.argv) > 2:
        cmd_start(sys.argv[2])
    elif cmd == 'finish':
        cmd_finish(sys.argv[2] if len(sys.argv) > 2 else '')
    elif cmd == 'done' and len(sys.argv) > 2:
        cmd_done(sys.argv[2])
    elif cmd == 'decide' and len(sys.argv) > 4:
        cmd_decide(sys.argv[2], sys.argv[3], sys.argv[4])
    elif cmd == 'state' and len(sys.argv) > 3:
        cmd_state(sys.argv[2], sys.argv[3])
    elif cmd == 'todo' and len(sys.argv) > 2:
        cmd_todo(sys.argv[2])
    else:
        print(f'Unknown command or missing args: {sys.argv[1:]}')
        print('Usage: tracker.py <status|start|finish|done|decide|state|todo> [args]')

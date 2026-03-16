import { writable, derived, get } from 'svelte/store';
import { spawnAgentSession, agentSessions, updateSessionStatus, type SessionStatus } from './agentTerminal';

export interface MissionTask {
	id: string;
	description: string;
	branchName: string;
	sessionId: string | null;
	agent: string;
	status: 'queued' | 'working' | 'done' | 'error' | 'reviewing';
	order: number;
}

export interface Mission {
	id: string;
	description: string;
	tasks: MissionTask[];
	status: 'planning' | 'running' | 'reviewing' | 'done' | 'cancelled';
	agent: string;
	maxParallel: number;
	createdAt: number;
}

export const missions = writable<Mission[]>([]);

export const activeMission = derived(
	missions,
	$missions => $missions.find(m => m.status === 'running' || m.status === 'reviewing') ?? null
);

export function createMission(description: string, tasks: Omit<MissionTask, 'id' | 'sessionId' | 'status' | 'order'>[], agent: string, maxParallel: number): Mission {
	const mission: Mission = {
		id: crypto.randomUUID(),
		description,
		tasks: tasks.map((t, i) => ({
			...t,
			id: crypto.randomUUID(),
			sessionId: null,
			status: 'queued' as const,
			order: i,
		})),
		status: 'planning',
		agent,
		maxParallel,
		createdAt: Date.now(),
	};
	missions.update(m => [...m, mission]);
	return mission;
}

export async function startMission(missionId: string) {
	missions.update(ms => ms.map(m =>
		m.id === missionId ? { ...m, status: 'running' as const } : m
	));

	await spawnNextTasks(missionId);
}

async function spawnNextTasks(missionId: string) {
	const ms = get(missions);
	const mission = ms.find(m => m.id === missionId);
	if (!mission || mission.status !== 'running') return;

	const activeTasks = mission.tasks.filter(t => t.status === 'working');
	const queuedTasks = mission.tasks.filter(t => t.status === 'queued');
	const slotsAvailable = mission.maxParallel - activeTasks.length;

	for (let i = 0; i < Math.min(slotsAvailable, queuedTasks.length); i++) {
		const task = queuedTasks[i];
		try {
			const sessionId = await spawnAgentSession(undefined, mission.agent);
			missions.update(ms => ms.map(m =>
				m.id === missionId ? {
					...m,
					tasks: m.tasks.map(t =>
						t.id === task.id ? { ...t, sessionId, status: 'working' as const } : t
					),
				} : m
			));
		} catch (e) {
			missions.update(ms => ms.map(m =>
				m.id === missionId ? {
					...m,
					tasks: m.tasks.map(t =>
						t.id === task.id ? { ...t, status: 'error' as const } : t
					),
				} : m
			));
		}
	}
}

export function markTaskDone(missionId: string, taskId: string) {
	missions.update(ms => ms.map(m => {
		if (m.id !== missionId) return m;
		const tasks = m.tasks.map(t =>
			t.id === taskId ? { ...t, status: 'done' as const } : t
		);
		const allDone = tasks.every(t => t.status === 'done' || t.status === 'error');
		return { ...m, tasks, status: allDone ? 'reviewing' as const : m.status };
	}));

	// Spawn next queued tasks
	spawnNextTasks(missionId);
}

export function cancelMission(missionId: string) {
	missions.update(ms => ms.map(m =>
		m.id === missionId ? { ...m, status: 'cancelled' as const } : m
	));
}

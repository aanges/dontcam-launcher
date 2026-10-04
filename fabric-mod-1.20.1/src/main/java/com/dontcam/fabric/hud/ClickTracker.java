package com.dontcam.fabric.hud;

import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.minecraft.client.MinecraftClient;

import java.util.ArrayDeque;
import java.util.Deque;

/** Rolling 1-second left/right click counters for the CPS HUD. */
public class ClickTracker {

    private static final long WINDOW_NS = 1000000000L;

    private final Deque<Long> left = new ArrayDeque<>();
    private final Deque<Long> right = new ArrayDeque<>();
    private boolean lastAttack = false;
    private boolean lastUse = false;

    private static ClickTracker instance;

    public static ClickTracker get() {
        return instance;
    }

    public static void init() {
        instance = new ClickTracker();
        ClientTickEvents.START_CLIENT_TICK.register(client -> {
            if (client.player == null || instance == null) {
                return;
            }
            long now = System.nanoTime();
            boolean attack = client.options.attackKey.isPressed();
            boolean use = client.options.useKey.isPressed();
            // Edge-triggered: count the press, not the hold.
            if (attack && !instance.lastAttack) {
                synchronized (instance.left) {
                    instance.left.addLast(now);
                }
            }
            if (use && !instance.lastUse) {
                synchronized (instance.right) {
                    instance.right.addLast(now);
                }
            }
            instance.lastAttack = attack;
            instance.lastUse = use;
        });
    }

    private static int prune(Deque<Long> queue, long now) {
        synchronized (queue) {
            while (!queue.isEmpty() && now - queue.peekFirst() > WINDOW_NS) {
                queue.pollFirst();
            }
            return queue.size();
        }
    }

    public int leftCps() {
        return prune(left, System.nanoTime());
    }

    public int rightCps() {
        return prune(right, System.nanoTime());
    }
}

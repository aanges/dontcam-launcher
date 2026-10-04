package com.dontcam.hud;

import net.minecraftforge.client.event.MouseEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;

import java.util.ArrayDeque;
import java.util.Deque;

/** Rolling 1-second left/right click counters for the CPS HUD. */
public class ClickTracker {

    private static final long WINDOW_NS = 1000000000L;

    private final Deque<Long> left = new ArrayDeque<Long>();
    private final Deque<Long> right = new ArrayDeque<Long>();

    @SubscribeEvent
    public void onMouse(MouseEvent event) {
        if (!event.buttonstate) {
            return;
        }
        long now = System.nanoTime();
        if (event.button == 0) {
            synchronized (left) {
                left.addLast(now);
            }
        } else if (event.button == 1) {
            synchronized (right) {
                right.addLast(now);
            }
        }
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

    private static ClickTracker instance;

    public static ClickTracker get() {
        return instance;
    }

    public ClickTracker() {
        instance = this;
    }
}

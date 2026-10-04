package com.dontcam.fabric.gui;

import net.minecraft.client.gui.widget.SliderWidget;
import net.minecraft.text.Text;

import java.util.function.DoubleConsumer;

/** 0..255 channel slider. Value changes call back live (no disk writes). */
public class DontCamSliderWidget extends SliderWidget {

    private final String label;
    private final DoubleConsumer onChange;

    public DontCamSliderWidget(int x, int y, int width, String label, int value, DoubleConsumer onChange) {
        super(x, y, width, 20, Text.literal(label + ": " + value), value / 255.0);
        this.label = label;
        this.onChange = onChange;
    }

    public void setChannel(int value) {
        this.value = Math.max(0, Math.min(255, value)) / 255.0;
        this.updateMessage();
    }

    public int getChannel() {
        return (int) Math.round(this.value * 255.0);
    }

    @Override
    protected void updateMessage() {
        this.setMessage(Text.literal(label + ": " + getChannel()));
    }

    @Override
    protected void applyValue() {
        if (onChange != null) {
            onChange.accept(this.value);
        }
    }
}

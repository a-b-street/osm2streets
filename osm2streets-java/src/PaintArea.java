package org.osm2streets;

import org.osm2streets.LatLon;

import java.util.List;

public class PaintArea {
    public List<LatLon> area;
    public String color;

    public PaintArea(List<LatLon> area, String color) {
        this.area = area;
        this.color = color;
    }
}

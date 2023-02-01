package org.osm2streets;

import org.osm2streets.LatLon;

import java.util.List;

public class Surface {
    public List<LatLon> area;
    public String material;

    public Surface(List<LatLon> area, String material) {
        this.area = area;
        this.material = material;
    }
}

#version 330 core
in vec4 FragPos;

uniform vec3 light_pos;
uniform float FAR_PLANE;

void main()
{
    float lightDistance = length(FragPos.xyz - light_pos);
    
    // map to [0;1] range by dividing by FAR_PLANE
    lightDistance = lightDistance / FAR_PLANE;
    
    // write this as modified depth
    gl_FragDepth = lightDistance;
}
#version 330 core
out vec4 FragColor;

in VS_OUT {
    vec3 FragPos;
    vec3 Normal;
    vec2 TexCoords;
} fs_in;

uniform sampler2D diffuseTexture;
uniform samplerCube depthMap;

uniform vec3 light_pos;
uniform vec3 viewPos;

uniform float FAR_PLANE;
uniform bool shadows;

float ShadowCalculation(vec3 fragPos)
{
    // get vector between fragment position and light position
    vec3 fragToLight = fragPos - light_pos;
    // ise the fragment to light vector to sample from the depth map    
    float closestDepth = texture(depthMap, fragToLight).r;
    // it is currently in linear range between [0,1], let's re-transform it back to original depth value
    closestDepth *= FAR_PLANE;
    // now get current linear depth as the length between the fragment and light position
    float currentDepth = length(fragToLight);
    // test for shadows
    float bias = 0.05; // we use a much larger bias since depth is now in [NEAR_PLANE, FAR_PLANE] range
    float shadow = 0.0;
    if (currentDepth -  bias > closestDepth ) {
	shadow =  1.0 ;
    }
    // display closestDepth as debug (to visualize depth cubemap)
    //FragColor = vec4(vec3(closestDepth / FAR_PLANE), 1.0);
        
    return shadow;
}

void main()
{           
    vec3 color = texture(diffuseTexture, fs_in.TexCoords).rgb;
    vec3 normal = normalize(fs_in.Normal);
    vec3 lightColor = vec3(0.6);
    // ambient
    vec3 ambient = 0.5 * color;
    // diffuse
    vec3 lightDir = normalize(light_pos - fs_in.FragPos);
    float diff = max(dot(lightDir, normal), 0.0);
    vec3 diffuse = diff * lightColor;
    // specular
    vec3 viewDir = normalize(viewPos - fs_in.FragPos);
    vec3 reflectDir = reflect(-lightDir, normal);
    float spec = 0.0;
    vec3 halfwayDir = normalize(lightDir + viewDir);  
    spec = pow(max(dot(normal, halfwayDir), 0.0), 64.0);
    vec3 specular = spec * lightColor;    
    // calculate shadow
    float shadow = ShadowCalculation(fs_in.FragPos) ; //: 0.0;                      
    vec3 lighting = (ambient + (1.0 - shadow) * (diffuse + specular)) * color;    
    
    FragColor = vec4(lighting, 1.0);
}

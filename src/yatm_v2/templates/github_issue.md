# {{ name  }}

{{ description }}

## Process
{% for step in steps %}
### Step {{ loop.index }}

#### Actions
    {% for action in step.action -%}
        {%- match action -%}
            {% when Action::StdIn with (terminal) %}
* StdIn: `{{ terminal.text }}`
            {% when Action::Image with (image_path) %}
* ![Image]({{ image_path }})
            {% when Action::Describe with (description) %}
* Description: {{ description }}
        {%- endmatch -%}
    {%- endfor %}

#### Expected Result
    {% for expect in step.expect -%}
        {%- match expect -%}
            {% when Expect::StdOut with (terminal) %}
* StdOut: {{ terminal.number }} - {{ terminal.text }}
            {% when Expect::StdErr with (terminal) %}
* StdErr: {{ terminal.number }} - {{ terminal.text }}
            {% when Expect::Image with (image) %}
* Image: {{ image }}
            {% when Expect::Describe with (description) %}
* Description: {{ description }}
        {%- endmatch -%}
    {%- endfor -%}
{%- endfor %}

Bottom business
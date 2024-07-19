Feature: Workspace icon

  Background:
    Given the Prose Pod API has started

  Rule: The API should warn if the workspace has not been initialized when getting the workspace icon

    Scenario: XMPP server and workspace not initialized
      Given the server config has not been initialized
       When a user gets the workspace icon
       Then the user should receive 'Server config not initialized'

    Scenario: XMPP server initialized but not the workspace
      Given the server config has been initialized
        And the workspace has not been initialized
       When a user gets the workspace icon
       Then the call should succeed
        And the response content type should be JSON
        And the returned workspace icon should be undefined

  Rule: A user can request the workspace icon

    Scenario: Get workspace icon after initializing
      Given the Prose Pod has been initialized
       When a user gets the workspace icon
       Then the call should succeed
        And the response content type should be JSON
        And the returned workspace icon should be undefined

    Scenario: Get workspace icon after setting it once
      Given the Prose Pod has been initialized
        And the workspace icon is "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"
       When a user gets the workspace icon
       Then the call should succeed
        And the response content type should be JSON
        And the returned workspace icon should be "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"

  Rule: An admin user can change the workspace icon

    Scenario: Change workspace icon
      Given the Prose Pod has been initialized
        And the workspace icon is "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"
       When a user sets the workspace icon to "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"
       Then the call should succeed
        And the response content type should be JSON
        And the returned workspace icon should be "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"
        And the workspace icon should be "iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAYAAABzenr0AAAABGdBTUEAALGPC/xhBQAAACBjSFJNAAB6JgAAgIQAAPoAAACA6AAAdTAAAOpgAAA6mAAAF3CculE8AAAAhGVYSWZNTQAqAAAACAAFARIAAwAAAAEAAQAAARoABQAAAAEAAABKARsABQAAAAEAAABSASgAAwAAAAEAAgAAh2kABAAAAAEAAABaAAAAAAAAAEgAAAABAAAASAAAAAEAA6ABAAMAAAABAAEAAKACAAQAAAABAAAAIKADAAQAAAABAAAAIAAAAABfvA/wAAAACXBIWXMAAAsTAAALEwEAmpwYAAACymlUWHRYTUw6Y29tLmFkb2JlLnhtcAAAAAAAPHg6eG1wbWV0YSB4bWxuczp4PSJhZG9iZTpuczptZXRhLyIgeDp4bXB0az0iWE1QIENvcmUgNi4wLjAiPgogICA8cmRmOlJERiB4bWxuczpyZGY9Imh0dHA6Ly93d3cudzMub3JnLzE5OTkvMDIvMjItcmRmLXN5bnRheC1ucyMiPgogICAgICA8cmRmOkRlc2NyaXB0aW9uIHJkZjphYm91dD0iIgogICAgICAgICAgICB4bWxuczp0aWZmPSJodHRwOi8vbnMuYWRvYmUuY29tL3RpZmYvMS4wLyIKICAgICAgICAgICAgeG1sbnM6ZXhpZj0iaHR0cDovL25zLmFkb2JlLmNvbS9leGlmLzEuMC8iPgogICAgICAgICA8dGlmZjpZUmVzb2x1dGlvbj43MjwvdGlmZjpZUmVzb2x1dGlvbj4KICAgICAgICAgPHRpZmY6UmVzb2x1dGlvblVuaXQ+MjwvdGlmZjpSZXNvbHV0aW9uVW5pdD4KICAgICAgICAgPHRpZmY6WFJlc29sdXRpb24+NzI8L3RpZmY6WFJlc29sdXRpb24+CiAgICAgICAgIDx0aWZmOk9yaWVudGF0aW9uPjE8L3RpZmY6T3JpZW50YXRpb24+CiAgICAgICAgIDxleGlmOlBpeGVsWERpbWVuc2lvbj4zMDA8L2V4aWY6UGl4ZWxYRGltZW5zaW9uPgogICAgICAgICA8ZXhpZjpDb2xvclNwYWNlPjE8L2V4aWY6Q29sb3JTcGFjZT4KICAgICAgICAgPGV4aWY6UGl4ZWxZRGltZW5zaW9uPjMwMDwvZXhpZjpQaXhlbFlEaW1lbnNpb24+CiAgICAgIDwvcmRmOkRlc2NyaXB0aW9uPgogICA8L3JkZjpSREY+CjwveDp4bXBtZXRhPgoMykTYAAALiklEQVRYCX1Xa4xU5Rl+vnObM/fZ6+yFZVlAuSgIsrUm2tatprRIjTZCtC2NLW350TRtGtuS+GeT/mjaNLY//KOxVlsTLWCKxUoBkVW8VFmUAu4KyH1Z2F2YnZ2ZM+d+Tp9vcBvUpmcycy5zzve+7/M+7/O+R+D/bFu2bDHWr1/vzd7ywl923IwguE03jOWKps2JgiAVA7GqKhYQXYxD/7CqBPvXPHj/+7PPfHqN2euzezF7cO1+cHBQWbp0qaDxMB6O9Vcu7N/k+f53rPLMymwmqzWndJjVs9CjOhyRgO3Rx1QBrlpAyQ58ROFBXVGe6V7a+cf+/n6fTqjr1q2LhBD095PbZxzYsiVW168Xobxt+3M77/N8PJrLFuYJXqqUS0gFdpS+ciSyT2+PrWo1niyXRKlShZLrE22LvqbMWXaHEpoFRBCw6vZp3dB+dv+GNdvlevv27dMGBgYCeTy7fcKBa2/Y+uwrj5kJ40e+78F13DCKfCQUH13WqHJ86FfYu98XYQsNKRFy+QKcuoWm+Eq8/M7vR8033gk3TiKRSKqJVBauZz923wN3/lgaffzxYX3Tpn5/1gF19mDfvpje9TW8e3Hb2zuaCq3fLlfLYRgEMZFTEfpKtnxc0cdfExMlR5x1fUxHNdQ9F1FECqgCgd4qyqWTSlv39Uom367EEIEfBFEh13brvfesv2XJjX3PP/zwN8N9g/u0Z4aeiaRtTf5I2AcGRMP4P18+/LdMJrd2ZnrSzReaEk5tBrZj88YAWe8cqqWTcDwdi+YvQqBmoGhJZNJZJBMmLNvHkeGtsGpVZBDyo2maKuJypWQ3NbetWb70Sy/S3NcHBgeCOGZcjEyRB9IJue3Z88FvafTeauWKq2nC0DQF6ZRJKDWYCR2RlsXxKQd6ywIU893o6+jDdb2L0VNciDnFJbiuZwnm9t4ORdUQR6FMHWw3EEIRyUq1bBdyzWu3Prvn99LW1q1bFblXDh48qEnS7d09+hVVS/y8VivFMSI1IosC14ZQFWSzWZhGAlMzPi5NT/NfE9WajUp1BrEfMv+sAtJWUwwkWQ0e+dKgO3+CIAA9AEvVrFvVIGGmfvrnP+1cIytsmHzQrpbJOtWN8Ou8mYEnPIelnRQEho+xvH3ULQdxEMFVNCTTXY3IqvU6fKZHFxp03cSZy+Nobi3CTJvIZJsRRjK7MkgBx3WQVlKCTsD3AhD2RxnlTiH6/QYMhrH5/lgYN1+8NE7OxGYcMnyJA2GsMp81y0Lg+8hrBro6OomKilQ+D50M94hANlsgQ0SDkM3Fechni0RE8IqUqZhV4BGtKlzX03zfCZPJ5KJtz+15iN41XKQh9XuqIIsDLwxkzgiZT3bXq2XwvAFhbNtoPjeGW+cuQCZJqMmPlCKg0oaqJNDXtxgtLZ0wSNeYUep6jISp8T8BjXyPqU+WVQMjpEaA+3CDdEC8+eap3pma96Hv+GYUeiFEpGoULN+tU9A8nobcR0jrKrRDryOaeBsX1CSdqzE9AqlUK5rbexEIncvFUFMZpOYtRmCmQVT5FUyZzeC4jpBVr7I4FNUPvFoqa6xkmpX+tvZ2U1Ejn3lnvUcsM5uusYr5URSSMJ1GMpOGuOlzmJn7eXjMaaSpsBxKcVBH1S7RyGUEiguz2A01meXzCpGLaEsgYehQeNzIN12itkA3zIxT8W9RatX6Cp/wKKqINMJGrAhrxDKKoGsCZjLBWhfMNcUmkUDzgmXMfQFSrLWY1XC5jFMnPuTiATI0rpOAkjsRnw8YTMhjwSA03WA4ET8kB+LQpG6YqdQyrVqtLHDJcOZb6PQSAckXuoQqgkcCecx9GPogOQhWCD2gKLFatEoJ6VyOuc7DKDQhYr4DPUkJllVDdSQCTIgEgmhI6BVWKp3iGZGOFCHUAH6vlsmkcslUmtGw1Mh026uRuPSaNyeYd8Eyi6VgRkYjLWbNhxNTYMwUF9NZ9wk6xI5ISyOjH6J9foDu7gXww5jPcRWWc8RAJBcU3h+RCyERltWhqGpO0ViUhkbGks4KvQ9D5jfwmAn6ykaj8XqS9nW40OozMK1JKEwLuufDaW6VjQaKXUdaTTBDOWzZ/lccOfovWJWLUAILSZ1EpYNpps+gQoqI4TAJAVPjua6QajutSriYfsLS8DYg5OznRMVAaFUwMX4aF8+NYmFXG2ok4InzF1DJszx9lmG9iiTrXOSKSLS34vCpsxCVJ9DduRD5fBvm9C5AobUbheYOJFJ5KqoK15elTRSicFqjxY+k6knehSGZRfIkJHPp/fSpYzg68h6OfPAaLh25jE0P/wAZGvKMJjZHH6aSRKU0A7QV0dI1Dyrngox9EYu7voD2lIGn/7GD6ggU2K47elZg4aKb0DNvSZzJtSsqg6Q2nNGCsP6e4zLS0FMpRFSIOqypMZw8O4LDh/fi/ZHzSHZ0YDIN5p5wJrKIWPtW5RIMEk9v60TqxtsR6hm0TI3gkXvWoc48F7MGVtywGO9emWaXBEbeGcLr7w6hraU93vDQZrV37vXkQXRIStUBi2GoiPMy8efPjGoHXt2GcTpRYV4Mwi41gb2C5CFJ61TH0KbcZtFaaEE214u4tQfVyTHkps6gI5OBYOMKIwdmSJ570yxbilWxD4qRxIFjI/GqE8fQ0Tlvui2Tf0e5665bJ+LA2yUrIfQ8/+z5k3jj4Ci8ZAGKTuqxKmzXQjYvcGDkKA6NDKPuTKMz3wS7YuH8xZMoXTiEd4d34+1L47hCHbGJJPsojXQRKY+VzeqXhc30drQ0y3TzG7yx+sHV5xviFMX+kw4Z7rpWYqYyE6XZS/yIo5isBsIuuZEwMnhp/wHCuIfya6JrTi+VsIbjH+zF0QM7cWriBM5TG8bGxjBx5TKLWKCHDjATVCzjKrlZWZcmS0KKWzqTepL/XFXHu7+xeo/vWruyubQyXZ7yWBB8gKyUlSxdpI741Iautlak2ek8RlLxKSqEWrD0BHmRVlQUc1lkCzmoOrnNB1lHKNBZGUzEIGS/W9Dbo0xOTry1dv3av3NlocihgAfsXPojrm3FXd1zzfExPsNr0hFOTI29PPcCB5XaZXhczKMy1qmSHlMkBxeP08/Lo0fx1kfHMGOVOAPUyZmY80KKJLQZUBhZrqO1shxjBD/hcuD4z060qT/ggfHltasPUhN+keN8R51QKL+hFIfGvMYfuY+ZX6GYNOpg8tIZuNY0DBJLapzcfA4swxdO4Up1HDU6ahL+RXN7wRmAAHhBJp1BoZDd/PwLLwwP3jGo0W4gn5SByoFU3LPhW78bv3DuyaVLi6rrliVkfsOFBhz0uyGvOmaqUxg7exw2+7vK4SQgMrLNpOnlJWJ3umahbFdhcBDom9Pha3ypyGbzLKjiU7t2vfob2sLg0KDkpcwuMDQ0FPPNRR0ZGYmPnji9Y9kNS+d4nt1ft6tqpOoO+wZH7FjElFFZip1Fjl6MPGAXVSnXslnV2fOnZipEI8REtYLACaJypeIMjx5KaIVOtbOj96nde4Y2Snsf25JhXXVAHkjjs06cPDO+o3/FSst2w7v80NI5PMi+5tBb2VKU1qYWSjWv0BgVm50ybKBQrnJ0C0WgqZp7tFTWHW9Kb2vriZtaezbv3D30S2lH2uBE3IhenjcQkAdyu8YJceLU2Ju3f/G2l2h7Lo1eFwV0xPMVn7NUS3Ozn9SMkGhwJhJs+WHguV40Q+grdZczSKT3tBTFvDmLdq1ctuKBJ59/aRuXF582Lm02OCYPrt1IDoVfOdY23ozvXn3bwPR06bu1avWr9Xq5bfGi+WjJ5sgcpzExObaDMsf1qaqPmocJQ0/u7mjrfHr362+9+vG6OtcL+W3Afq2t/+nA7A2rVq3S+d7w3/e4jRs3Np8/PbqiqWAuz+pKLzt5Tr6DcHCtVGZqpyPdONxdLP77D09vL8+u8UOu8cTBg5Lkkuyf2f4DAeIJqxFEdfQAAAAASUVORK5CYII="

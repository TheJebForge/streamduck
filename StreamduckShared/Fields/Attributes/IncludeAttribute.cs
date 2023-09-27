using System;

namespace Streamduck.Fields.Attributes; 

/**
 * Include non-public property in UI
 */
[AttributeUsage(AttributeTargets.Property)]
public class IncludeAttribute : Attribute { }